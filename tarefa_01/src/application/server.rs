use std::{
    net::UdpSocket, thread, time::Duration
};

use crate::constants::*;
pub struct Server;

impl Server{
    
    pub fn new() -> Server{
        Server{}
    }

    pub fn run(&mut self){
        println!("Initializing Server");
        let socket = UdpSocket::bind(SERVER_ADDRESS).expect("Failed to bind to address");
        let mut buffer: [u8; 1024] = [0; 1024];

        let mut i: u32 = 0;
        loop{
            let msg_string = format!("Hello from server! {i}");
            let msg_bytes = msg_string.as_bytes();
            buffer[..msg_bytes.len()].copy_from_slice(msg_bytes);

            if let Ok(bytes) = socket.send_to(&buffer[..msg_bytes.len()], CLIENT_ADDRESS){
                println!("Sent {bytes} bytes to {CLIENT_ADDRESS}");
            }

            i = i.saturating_add(1);
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

