use std::{
    collections::HashSet, fs, io::Error, net::UdpSocket, sync::{mpsc, Arc, Mutex}, thread, time::Duration
};

use crate::{application::ztp::ZTPRequestCode, constants::*};
use crate::application::ztp;
use thread_pool::ThreadPool;

use super::ztp::{ZTPMetadata, ZTPResponse, ZTPResponseCode, ZTPResponseData, ZTPRequest};

mod thread_pool;

/*================================================= SERVER ============================================================= */

pub struct Server{
    ports_in_use: HashSet<&'static str>,
    available_ports: HashSet<&'static str>,
    socket: UdpSocket,
    pool: ThreadPool,
    tx_buff: [u8; BUFF_SIZE],
    rx_buff: [u8; BUFF_SIZE],
}

impl Server{
    pub fn new() -> Server{
        let ports_in_use = HashSet::with_capacity(THREAD_POOL_SIZE);
        let mut available_ports: HashSet<&str> = HashSet::with_capacity(UDP_SOCKETS.len());
        let socket = UdpSocket::bind(SERVER_ADDRESS).unwrap();
        let pool = ThreadPool::new(THREAD_POOL_SIZE);
        for socket in UDP_SOCKETS.iter(){
            available_ports.insert(socket);
        }
        Server{
            ports_in_use,
            available_ports,
            socket,
            pool,
            tx_buff: [0u8; BUFF_SIZE],
            rx_buff: [0u8; BUFF_SIZE],
        }
    }

    pub fn run(&mut self){
        println!("Initializing Server");
        let (sender, receiver) = mpsc::channel::<String>();
        let sender = Arc::new(Mutex::new(sender));

        loop{
            //check for incomming requests
            if let Ok((bytes, addr)) = self.socket.recv_from(&mut self.rx_buff){
                if let Some(req) = parse_request(& self.rx_buff[..bytes]){
                    match req.get_code(){
                        ZTPRequestCode::Conn => {
                            let client_addr = format!("{addr}");
                            self.resolve_conn_req(
                                client_addr,
                                Arc::clone(&sender),
                            );
                        },
                        _ => {

                        },
                    }
                }
            }

            //clear finished requests 
            while let Ok(msg) = receiver.try_recv(){
                println!("Removing address {msg} from set");
                self.free_socket(&msg);
            }
        }
    }

    fn resolve_conn_req(
        &mut self, 
        addr: String, 
        sender: Arc<Mutex<mpsc::Sender<String>>>, 
    ){
        let free_socket = self.get_available_socket().unwrap();
        self.take_socket(free_socket);
        let sender_clone = Arc::clone(&sender);
        self.pool.execute(||{
            handle_connection(
                addr, 
                free_socket.to_string(), 
                sender_clone
            );
        });
    }

    fn send_response(&mut self, addr: &str ,res: ZTPResponse) -> Result<usize, Error>{
        let byte_count = ZTPResponse::encode_into_slice(res, &mut self.tx_buff).unwrap();
        self.socket.send_to(&self.tx_buff[..byte_count], addr)
    }

    fn get_available_socket(&self) -> Option<&'static str>{
        self.available_ports.iter().next().map(|v| *v)
    }

    fn free_socket(&mut self, addr: &str){
        self.ports_in_use.remove(addr);
        let static_addr = UDP_SOCKETS
            .iter()
            .copied()
            .find(|v| { *v == addr})
            .unwrap();
        self.available_ports.insert(static_addr);
    }

    fn take_socket(&mut self, addr: &str){
        self.available_ports.remove(addr);
        let static_addr = UDP_SOCKETS
            .iter()
            .copied()
            .find(|s| *s == addr)
            .unwrap();
        self.ports_in_use.insert(static_addr);
    }
}

fn handle_connection(
    client_addr: String,
    socket_addr: String, 
    sender: Arc<Mutex<mpsc::Sender<String>>>
){
    let socket = UdpSocket::bind(&socket_addr).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.connect(&client_addr).unwrap();
    println!("Starting job for addr: {client_addr}");
    let mut end_request = false;
    let mut rx_buff: [u8; 4096] = [0; 4096];
    let mut tx_buff: [u8; 4096] = [0; 4096];

    
    let accepted = accept_connection(
        &socket,
        &client_addr,
        &mut tx_buff,
        &mut rx_buff
    );
    println!("Connection Accepted");

    if !accepted{
        println!("Failed to establish connection with {client_addr}, returning");
        end_request = true;
    }
    
    while !end_request{
        while peek_request(&socket, &mut rx_buff).is_err(){}
        
        let bytes = get_request(&socket, &mut rx_buff).unwrap();

        let req = parse_request(&rx_buff[..bytes]);
        if req.is_none(){
            end_request = true;
            continue;
        }
        let req = req.unwrap();

        // drain_socket(&socket, &addr);
        println!("Client requested {}", req.get_resource());
        let res_buff;
        match get_resource(req.get_resource()){
            Ok(buff)  => {res_buff = buff;},
            Err(_) =>{
                println!("Resource does not exist!");
                send_not_found(&socket, &client_addr);
                end_request = true;
                continue;
            }
        }
        let metadata = ZTPResponse::new(
            ZTPResponseCode::Metadata,
            Some(ZTPResponseData::Metadata(
                ZTPMetadata::from_bytes(&res_buff)
            )),
            None
        );
        println!("Sending Metadata to {client_addr}");
        send_metadata(&socket, &client_addr, metadata);
        println!("Sending Resource to {client_addr}");
        send_resource(&socket, &client_addr, &res_buff);
        end_request = true;
    }
    println!("Sending EOR to {client_addr}");
    send_end_of_req(&socket, &client_addr);
    thread::sleep(Duration::from_millis(TTL_MILLIS));

    println!("Finishing job for address {client_addr}");
    sender.lock().unwrap().send(socket_addr).unwrap();
}

fn parse_request(buffer: &[u8]) -> Option<ZTPRequest>{
    if let Ok(req) = ZTPRequest::decode_from_slice(buffer){
       return Some(req.0); 
    }
    None
}

fn get_request(socket: &UdpSocket, buffer: &mut[u8]) -> Result<usize, Error>{
    socket.recv(buffer)
}

fn peek_request(socket: &UdpSocket, buffer: &mut[u8]) -> Result<usize, Error>{
    socket.peek(buffer)
}

fn get_resource(resource_name: &str) -> Result<Vec<u8>, Error>{ 
    let path = format!("./resources/{resource_name}");
    fs::read(&path)
}

fn send_not_found(socket: &UdpSocket, addr: &str) -> usize{
    let not_found_res = ZTPResponse::new(ZTPResponseCode::NotFound, None, None);
    let vec = ZTPResponse::encode_to_vec(
        not_found_res,
    ).unwrap();

    socket.send_to(&vec, addr).unwrap()
}

fn send_end_of_req(socket: &UdpSocket, addr: &str) -> usize{
    let end_of_req = ZTPResponse::new(ZTPResponseCode::EndRequest, None, None);
    let vec = ZTPResponse::encode_to_vec(
        end_of_req, 
    ).unwrap();

    socket.send_to(&vec, addr).unwrap()
}

fn send_metadata(
    socket: &UdpSocket,
    addr: &str,
    metadata: ZTPResponse 
) -> bool{
    let mut tx_buff: [u8; 2048] = [0; 2048];
    let mut rx_buff: [u8; 4096] = [0; 4096];
    let bytes = ZTPResponse::encode_into_slice(
        metadata, 
        &mut tx_buff, 
    ).unwrap();

    socket.send_to(&tx_buff[..bytes], addr).unwrap();
    println!("Sent Metadata, Waiting for ACK...");

    let mut tries = 0;
    while peek_request(socket, &mut rx_buff).is_err(){
        println!("Waiting, try {tries}");
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;
        if tries > MAX_RETRIES{
            return false;
        }
    }
    println!("Metadata ACK received!");
    let response = get_response(socket, &mut rx_buff);
    return true;
}

fn send_resource(
    socket: &UdpSocket,
    addr: &str,
    res_buff: &[u8],
){
    let size = res_buff.len();
    println!("Resource Size: {size}");
        
    let mut tx_buffer: [u8; 4096] = [0; 4096];
    let mut rx_buffer: [u8; 4096] = [0; 4096];
    let mut start = 0;
    let mut pkg_id = 0;
    while start <= size{
        let end = size.min(start + DATA_PIECE_SIZE);

        let response = ZTPResponse::new(
            ztp::ZTPResponseCode::Data, 
            Some(ZTPResponseData::Bytes(res_buff[start..end].to_vec())),
            Some(pkg_id),
        );

        let res_size = ZTPResponse::encode_into_slice(
            response,
            &mut tx_buffer,
        ).unwrap();

        let mut tries = 0;
        let mut package_finished = false;
        while !package_finished{
            println!("Sending Data Piece to {addr}, start: {start}, try = {tries}");
            println!("Sending {res_size} bytes");
            send_data_piece(socket, addr, &tx_buffer[..res_size]);
            thread::sleep(Duration::from_millis(TTL_MILLIS));
            
            let is_ack;
            if let Some(res) = get_response(socket, &mut rx_buffer){
                println!("Received Response from {addr}");
                is_ack = res.is_ack(); 
            } else{
                is_ack = false;
            }

            package_finished = tries >= 10 || is_ack;
            tries += 1;
        }

        if tries >= 10 {return;}
        
        start += DATA_PIECE_SIZE;
        pkg_id += 1;
    }

    let end_of_req = ZTPResponse::new(ZTPResponseCode::EndRequest, None, None);

    let end_of_req_size = ZTPResponse::encode_into_slice(
        end_of_req,
        &mut tx_buffer,
    ).unwrap();

    send_data_piece(socket, addr, &tx_buffer[..end_of_req_size]);

}

fn send_data_piece(
    socket: &UdpSocket,
    addr: &str,
    buff: &[u8],
){
    socket.send_to(buff, addr);
}

fn get_response(
    socket: &UdpSocket,
    rx_buff: &mut [u8],
) -> Option<ZTPResponse>{ 
    match socket.recv(rx_buff){
        Ok(bytes) =>{
            println!("Received {bytes} bytes");
            println!("Received bytes (hex): {:02x?}", &rx_buff[..bytes]);
            let response: ZTPResponse = ZTPResponse::decode_from_slice(
                &rx_buff[..bytes],
            ).unwrap().0;

            Some(response)
        },
        Err(_) => None
    }

}


fn send_nack(socket: &UdpSocket, addr: &str, tx_buff: &mut [u8]) -> usize{
    println!("Sending NACK");
    let nack = ZTPResponse::new(
        ZTPResponseCode::Nack,
        None,
        None
    );
    let bytes = ZTPResponse::encode_into_slice(nack, tx_buff).unwrap();
    socket.send(&tx_buff[..bytes]).unwrap()
}

fn accept_connection(
    socket: &UdpSocket,
    client_addr: &str,
    tx_buff: &mut [u8],
    rx_buff: &mut [u8]
) -> bool{
    let conn_res = ZTPResponse::new(
        ZTPResponseCode::ConnAccepted,
        None,
        None
    );

    let bytes_written = ZTPResponse::encode_into_slice(conn_res, tx_buff).unwrap();
    let mut tries = 0;
    while tries < MAX_RETRIES{
        send_data_piece(socket, client_addr, &tx_buff[..bytes_written]);
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        
        let ack = get_response(socket, rx_buff);
        if ack.is_none(){
            tries += 1;
            continue;
        }

        if ack.unwrap().is_ack(){return true;}
        
        tries += 1;

    }
    
    false
}

