use std::{
    collections::HashMap, env, net::UdpSocket, thread, time::Duration
};

const CLIENT_ADDRESS: &'static str = "127.0.0.1:4242"; 
const SERVER_ADDRESS: &'static str = "127.0.0.1:34254";

fn main() {
    let var_map = collect_vars();
    if let Some(role) = var_map.get("role"){
        match role.as_str(){
            "server" => server(),
            _ => client()
        }
    }
    else{
       client(); 
    }
}

fn server(){
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

fn client(){
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

fn collect_vars() -> HashMap<String, String>{
    env::args()
        .skip(1)
        .filter_map(|arg|{
            if let Some((key, value)) = arg.split_once('='){
                return Some((key.to_string(), value.to_string())); 
            }
            None
        })
        .collect()
}
