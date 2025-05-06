use std::{
    collections::HashSet, fs, io::Error, net::UdpSocket, sync::{mpsc, Arc, Mutex}, thread, time::Duration
};

use crate::constants::*;
use crate::application::ztp;

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

    while !end_request{
        while peek_request(&socket, &addr, &mut rx_buff).is_err(){}
        
        let bytes = get_request(&socket, &addr, &mut rx_buff).unwrap();

        let (_, resource_name) = parse_request(&rx_buff[..bytes]);

        let res_buff;
        match get_resource(&resource_name){
            Ok(buff)  => {res_buff = buff;},
            Err(_) =>{
                send_not_found(&socket, &addr, &resource_name);
                end_request = true;
                continue;
            }
        }
        let metadata = format!("bytes={}", res_buff.len()); 
        send_metadata(&socket, &addr, &metadata);
        send_resource(&socket, &addr, &res_buff);
        send_end_of_req(&socket, &addr);
    }

    println!("Finishing job for address {addr}");
    sender.lock().unwrap().send(addr).unwrap();
}

fn parse_request(buffer: &[u8]) -> (String, String){
    let req_str = String::from_utf8_lossy(buffer);

    if let Some((method, resource)) = req_str.split_once(' '){
        return (method.to_string(), resource.to_string());
    }

    (String::from(""), String::from(""))

}

fn get_request(socket: &Arc<Mutex<UdpSocket>>, addr: &str, buffer: &mut[u8]) -> Result<usize, Error>{
    let locked_socket = socket.lock().unwrap();
    locked_socket.send_to(buffer, addr)
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

fn send_not_found(socket: &Arc<Mutex<UdpSocket>>, addr: &str ,res: &str) -> usize{
    let msg_str = format!("NOT_FOUND {res}");
    let msg_buff = msg_str.as_bytes();

    socket.lock().unwrap().send_to(msg_buff, addr).unwrap()
}

fn send_end_of_req(socket: &Arc<Mutex<UdpSocket>>, addr: &str) -> usize{
    let msg_bytes = "END_REQUEST".as_bytes();
    let locked_socket = socket.lock().unwrap();

    locked_socket.send_to(msg_bytes, addr).unwrap()
}

fn send_metadata(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    metadata: &str 
) -> bool{
    let msg_str = format!("METADATA {metadata}");
    let msg_bytes = msg_str.as_bytes();
    let mut rx_buff: [u8; 1024] = [0; 1024];

    let locked_socket = socket.lock().unwrap();
    locked_socket.send_to(msg_bytes, addr).unwrap();

    let mut tries = 0;
    while peek_request(socket, addr, &mut rx_buff).is_err(){
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;
        if tries > MAX_RETRIES{
            return false;
        }
    }
    return true;
}

fn send_resource(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    res_buff: &[u8],
){
   for data_piece in res_buff.iter().step_by(DATA_PIECE_SIZE){
         
   }
}

fn send_data_piece(
    socket: &Arc<Mutex<UdpSocket>>,
    addr: &str,
    buff: &[u8],
){

}
