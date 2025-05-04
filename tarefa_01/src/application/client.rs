use std::net::UdpSocket; 
use crate::constants::*;

pub struct Client;

impl Client{
    
    pub fn new() -> Client{
        Client{}
    }

    pub fn run(&mut self){
        println!("Initializing Client");
        let socket = UdpSocket::bind(CLIENT_ADDRESS).expect("Failed initialize Client");
        let mut buffer: [u8; 1024] = [0; 1024];


        loop{
            if let Ok((bytes, addr)) = socket.recv_from(&mut buffer){
                if bytes > 0{
                    let msg = String::from_utf8_lossy(&buffer[..bytes]);
                    println!("Received Message from {addr}");
                    println!("Mesage: {msg}\n");
                }
            }    
        }
    }
}
