use std::{io::{Read, Write}, net::TcpStream, time::Duration};


pub struct Client{
   stream: TcpStream 
}

impl Client{
    pub fn new(addr: &str) -> Client{
        let stream = TcpStream::connect(addr).expect("Failed to connect to server on address: {addr}");
        Client{stream}
    }

    pub fn run(&mut self){
        let mut tries = 0;
        self.stream.set_read_timeout(
            Some(Duration::from_millis(100))
        ).unwrap();
        let mut str_buff = String::with_capacity(100);
        loop{
           match self.stream.read_to_string(&mut str_buff){
                Ok(_) =>{
                    tries = 0;
                    println!("Received message: {str_buff}");
                    str_buff.clear();
                },
                Err(_) =>{
                    tries += 1;
                }
            }

            if tries > 10{
                tries = 0;
                let request = "Hi".to_string();
                self.stream.write(request.as_bytes()).expect("Failed to send high message");
            }
        }
    }
}
