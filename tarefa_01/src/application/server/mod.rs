use std::{
    collections::HashSet, net::UdpSocket, sync::{Arc, Mutex, mpsc}
};

use crate::constants::*;

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
        
        socket.lock().unwrap().set_nonblocking(true);
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
    let mut stop = false;
    let mut rx_buff: [u8; 4096] = [0; 4096];
    let hi_str = "Hello World!".as_bytes();

    while !stop{
        let socket_locked = socket.lock().unwrap();
        socket_locked.connect(&addr);
        if let Ok(_) = socket_locked.recv(&mut rx_buff){
            socket_locked.send(hi_str).unwrap();
            stop = true;
        }
    }
    
    println!("Finishing job for address {addr}");
    sender.lock().unwrap().send(addr);
}
