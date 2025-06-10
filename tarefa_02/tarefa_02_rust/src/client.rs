use std::{io::{Read, Write}, net::TcpStream, time::Duration};


pub struct Client{
   addr: String 
}

impl Client{
    pub fn new(addr: &str) -> Client{
        Client{addr: addr.to_string()}
    }

    pub fn run(& self){
        let mut stream = TcpStream::connect(&self.addr).expect("Failed to connect to server on address: {addr}");
        println!("Local address: {}", stream.local_addr().unwrap());
        let mut tries = 0;
        let request = "Hi".to_string();
        stream.set_read_timeout(
            Some(Duration::from_millis(1000))
        ).unwrap();
        stream.set_nodelay(true).unwrap();

        let mut rx_buff = [0u8; 4096];
        stream.write(request.as_bytes()).expect("Failed to send high message");
        println!("Sent message");
        loop{
           match stream.read(&mut rx_buff){
                Ok(read) =>{
                    if read == 0{
                        println!("Connection closed by server");
                        break;
                    }
                    let msg = String::from_utf8_lossy(&rx_buff[..read]).to_string();
                    tries = 0;
                    println!("Received message: {msg}");
                },
                Err(e) =>{
                    println!("Error receiving message: {e}");
                    tries += 1;
                }
            }

            if tries > 10{
                break;
            }
        }
    }
}
