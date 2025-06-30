use std::{io::Write, net::{TcpListener, TcpStream}, thread, time::Duration};


pub struct Server{
    listener: TcpListener,
}

impl Server{
    pub fn new(addr: &str) -> Server{
        let listener = TcpListener::bind(addr).expect("Failed to create TCP listener");
        Server{listener}
    }

    pub fn run(&mut self){
        println!("Starting server listening on address {}", self.listener.local_addr().unwrap());
        for stream in self.listener.incoming(){
            match stream{
                Ok(stm) => {
                    thread::spawn(move||{
                        handle_connection(stm);
                    });
                }, 
                Err(err) => eprintln!("Failed to stabilish connection: {err}")
            }
        }
        println!("Shutting down server bind to port {}", self.listener.local_addr().unwrap());
    }
}


fn handle_connection(mut stream: TcpStream){
    stream.set_nodelay(true).unwrap();
    println!("Received stream from remote address: {}", stream.peer_addr().unwrap());
    for i in 0..10{
        let response = format!("Hello number {i}!");
        println!("Sending message: {response}");
        stream.write(response.as_bytes()).unwrap();
        thread::sleep(Duration::from_millis(900));
    }
}
