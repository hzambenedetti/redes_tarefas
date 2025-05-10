use std::{fs, net::UdpSocket, thread, time::Duration}; 
use crate::constants::*;

use bincode::config;

use super::ztp::{
    ZTPMetadata, ZTPResponse, ZTPResponseCode, ZTPResponseData
};

pub struct Client;

impl Client{
    
    pub fn new() -> Client{
        Client{}
    }

    pub fn run(&mut self){
        println!("Initializing Client");
        let socket = UdpSocket::bind(CLIENT_ADDRESS).expect("Failed initialize Client");
        socket.connect(SERVER_ADDRESS).unwrap();
        socket.set_nonblocking(true).unwrap();

        let metadata = receive_metadata(&socket);
        if metadata.is_none() {return}

        let resource = receive_resource(&socket, metadata.unwrap());
        if resource.is_none() {return}
        let save_path = format!("{CLIENT_DIR_PATH}/{RES_NAME}");
        
        fs::write(&save_path, &resource.unwrap());
    }
}

fn receive_metadata(socket: &UdpSocket) -> Option<ZTPMetadata>{
    let mut rx_buff = [0u8; 4096];
    let mut tries = 0;

    while tries <= MAX_RETRIES{
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;
        if let Err(_) = socket.peek(&mut rx_buff){
            continue;
        }
        
        let bytes = socket.recv(&mut rx_buff).unwrap();
        if let Some(res) =  parse_response(&rx_buff[..bytes]){
            return extract_metadata(res);
        }
    }

    None
}

fn receive_resource(socket: &UdpSocket, metadata: ZTPMetadata) -> Option<Vec<u8>>{
    let mut tx_buff = [0u8; 4096];
    let mut rx_buff = [0u8; 4096];
    let mut res_buff = Vec::with_capacity(metadata.size()); 
    let mut tries: usize = 0;
    let mut res_code = ZTPResponseCode::Data;

    while res_code == ZTPResponseCode::EndRequest{
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;

        if let Err(_) = socket.peek(&mut rx_buff){
            if tries > MAX_RETRIES { return None}
            continue;
        }

        let bytes = socket.recv(&mut rx_buff).unwrap();
        
        if let Some(response) = parse_response(&rx_buff[..bytes]){
            res_code = response.get_code();
            if res_code == ZTPResponseCode::Data{
                let data = response.get_bytes().unwrap();
                copy_data(&mut res_buff, data);
                send_ack(socket, &mut tx_buff);
            }
        }
        else{
            send_nack(socket, &mut tx_buff);
        }
    }

   Some(res_buff) 
}

fn parse_response(
    buffer: &[u8]
) -> Option<ZTPResponse>{
    if let Ok(res) = bincode::decode_from_slice(buffer, bincode::config::standard()){
        return Some(res.0);
    }
    None
}

fn send_ack(socket: &UdpSocket, tx_buff: &mut [u8]) -> usize{
    let ack = ZTPResponse::new(
        ZTPResponseCode::Ack,
        None
    );
    let bytes = bincode::encode_into_slice(ack, tx_buff, config::standard()).unwrap();
    socket.send(&tx_buff[..bytes]).unwrap()
}


fn send_nack(socket: &UdpSocket, tx_buff: &mut [u8]) -> usize{
    let nack = ZTPResponse::new(
        ZTPResponseCode::Nack,
        None
    );
    let bytes = bincode::encode_into_slice(nack, tx_buff, config::standard()).unwrap();
    socket.send(&tx_buff[..bytes]).unwrap()
}

fn copy_data(res_buff: &mut Vec<u8>, data: &[u8]) -> usize{
    let initial_len = res_buff.len();
    for &byte in data.iter(){
        res_buff.push(byte);
    }

    res_buff.len() - initial_len
}

fn extract_metadata(response: ZTPResponse) -> Option<ZTPMetadata>{
    if let Some(ZTPResponseData::Metadata(metadata)) = response.get_data(){
       return Some(*metadata);
    }
    None    
}
