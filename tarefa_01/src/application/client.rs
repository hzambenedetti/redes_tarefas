use std::{net::UdpSocket, thread, time::Duration}; 
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
        
        let std_req = "GET ANY".as_bytes();

        loop{
            socket.send_to(std_req, SERVER_ADDRESS).unwrap(); 
           

            if let Ok((bytes, addr)) = socket.recv_from(&mut buffer){
                if bytes > 0{
                    let msg = String::from_utf8_lossy(&buffer[..bytes]);
                    println!("Received Message from {addr}");
                    println!("Mesage: {msg}\n");
                }
            }

            thread::sleep(Duration::from_millis(1000));
        }
    }
}
