use std::{
    collections::HashSet, fs, io::Error, net::UdpSocket, sync::{mpsc, Arc, Mutex}, thread, time::Duration
};

use crate::constants::*;
use crate::application::ztp;

use super::ztp::{ZTPMetadata, ZTPResponse, ZTPResponseCode, ZTPResponseData, ZTPRequest};

mod thread_pool;

/*================================================= SERVER ============================================================= */

pub struct Server{
    connections: HashSet<String>, 
}

impl Server{
    
    pub fn new() -> Server{
        let connections = HashSet::with_capacity(THREAD_POOL_SIZE);
        Server{connections}
    }

    pub fn run(&mut self){
        println!("Initializing Server");
        let socket = Arc::new(Mutex::new(UdpSocket::bind(SERVER_ADDRESS).expect("Failed to bind to address")));
        let pool = thread_pool::ThreadPool::new(THREAD_POOL_SIZE);
        let mut buffer: [u8; 4096] = [0; 4096];
        
        let (sender, receiver) = mpsc::channel::<String>();

        let sender = Arc::new(Mutex::new(sender));
        
        socket.lock().unwrap().set_nonblocking(true).unwrap();
        loop{
            //check for incomming requests
            if let Ok((_, addr)) = socket.lock().unwrap().peek_from(&mut buffer){
                let addr_str = format!("{addr}");
                if !self.connections.contains(&addr_str){
                   self.connections.insert(addr_str.clone());
                    let socket_clone = Arc::clone(&socket);
                    let sender_clone = Arc::clone(&sender);
                    pool.execute(move ||{
                        handle_connection(
                            addr_str,
                            socket_clone,
                            sender_clone
                        )
                    });
                }
            }

            //clear finished requests 
            while let Ok(msg) = receiver.try_recv(){
                println!("Removing address {msg} from set");
                self.connections.remove(&msg);
            }
        }
    }
}

fn handle_connection(
    addr: String,
    socket: Arc<Mutex<UdpSocket>>,
    sender: Arc<Mutex<mpsc::Sender<String>>>
){
    println!("Starting job for addr: {addr}");
    let mut end_request = false;
    let mut rx_buff: [u8; 4096] = [0; 4096];

    println!("Draning Socket for addr: {addr}");

    while !end_request{
        while peek_request(&socket, &addr, &mut rx_buff).is_err(){}
        
        let bytes = get_request(&socket, &addr, &mut rx_buff).unwrap();

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
                send_not_found(&socket, &addr);
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
        println!("Sending Metadata to {addr}");
        dbg!(&metadata);
        send_metadata(&socket, &addr, metadata);
        println!("Sending Resource to {addr}");
        send_resource(&socket, &addr, &res_buff);
        end_request = true;
    }
    println!("Sending EOR to {addr}");
    send_end_of_req(&socket, &addr);
    thread::sleep(Duration::from_millis(TTL_MILLIS));
    drain_socket(&socket, &addr);

    println!("Finishing job for address {addr}");
    sender.lock().unwrap().send(addr).unwrap();
}

fn parse_request(buffer: &[u8]) -> Option<ZTPRequest>{
    if let Ok(req) = ZTPRequest::decode_from_slice(buffer){
       return Some(req.0); 
    }
    None
}

fn get_request(socket: &Arc<Mutex<UdpSocket>>, addr: &str, buffer: &mut[u8]) -> Result<usize, Error>{
    let locked_socket = socket.lock().unwrap();
    locked_socket.connect(addr).unwrap();
    locked_socket.recv(buffer)
}

fn peek_request(socket: &Arc<Mutex<UdpSocket>>, addr: &str, buffer: &mut[u8]) -> Result<usize, Error>{
    let locked_socket = socket.lock().unwrap();
    locked_socket.connect(addr).unwrap();
    locked_socket.peek(buffer)
}

fn get_resource(resource_name: &str) -> Result<Vec<u8>, Error>{ 
    let path = format!("./resources/{resource_name}");
    fs::read(&path)
}

fn send_not_found(socket: &Arc<Mutex<UdpSocket>>, addr: &str) -> usize{
    let not_found_res = ZTPResponse::new(ZTPResponseCode::NotFound, None, None);
    let vec = ZTPResponse::encode_to_vec(
        not_found_res,
    ).unwrap();

    socket.lock().unwrap().send_to(&vec, addr).unwrap()
}

fn send_end_of_req(socket: &Arc<Mutex<UdpSocket>>, addr: &str) -> usize{
    let end_of_req = ZTPResponse::new(ZTPResponseCode::EndRequest, None, None);
    let vec = ZTPResponse::encode_to_vec(
        end_of_req, 
    ).unwrap();

    let locked_socket = socket.lock().unwrap();
    locked_socket.send_to(&vec, addr).unwrap()
}

fn send_metadata(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    metadata: ZTPResponse 
) -> bool{
    let mut tx_buff: [u8; 2048] = [0; 2048];
    let mut rx_buff: [u8; 4096] = [0; 4096];
    let bytes = ZTPResponse::encode_into_slice(
        metadata, 
        &mut tx_buff, 
    ).unwrap();

    let locked_socket = socket.lock().unwrap();
    locked_socket.send_to(&tx_buff[..bytes], addr).unwrap();
    drop(locked_socket);
    println!("Sent Metadata, Waiting for ACK...");

    let mut tries = 0;
    while peek_request(socket, addr, &mut rx_buff).is_err(){
        println!("Waiting, try {tries}");
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;
        if tries > MAX_RETRIES{
            return false;
        }
    }
    println!("Metadata ACK received!");
    let response = get_response(socket, addr, &mut rx_buff);
    return true;
}

fn send_resource(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    res_buff: &[u8],
){
    let size = res_buff.len();
    println!("Resource Size: {size}");
        
    let mut tx_buffer: [u8; 4096] = [0; 4096];
    let mut rx_buffer: [u8; 4096] = [0; 4096];
    let mut start = 0;
    while start <= size{
        let end = size.min(start + DATA_PIECE_SIZE);

        let response = ZTPResponse::new(
            ztp::ZTPResponseCode::Data, 
            Some(ZTPResponseData::Bytes(res_buff[start..end].to_vec())),
            Some(0),
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
            if let Some(res) = get_response(socket,addr, &mut rx_buffer){
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
    }

    let end_of_req = ZTPResponse::new(ZTPResponseCode::EndRequest, None, None);

    let end_of_req_size = ZTPResponse::encode_into_slice(
        end_of_req,
        &mut tx_buffer,
    ).unwrap();

    send_data_piece(socket, addr, &tx_buffer[..end_of_req_size]);

}

fn send_data_piece(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    buff: &[u8],
){
    let locked_socket = socket.lock().unwrap();
    locked_socket.send_to(buff, addr);
}

fn get_response(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    rx_buff: &mut [u8],
) -> Option<ZTPResponse>{ 
    let locked_socket = socket.lock().unwrap();
    locked_socket.connect(addr);
    match locked_socket.recv(rx_buff){
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

fn drain_socket(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
){
    let mut drain_buff = [0u8; 4096];
    let locked_socket = socket.lock().unwrap();
    locked_socket.connect(addr).unwrap();

    while let Ok(bytes) = locked_socket.recv(&mut drain_buff){
        println!("Received bytes (hex): {:02x?}", &drain_buff[..bytes]);
    }
    
    println!("Finished Draining Socket");
}
