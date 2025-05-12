use std::{fs, net::UdpSocket, thread, time::Duration, collections::HashSet}; 
use xxhash_rust::xxh3;
use rand::prelude::*;

use crate::constants::*;

use super::ztp::{
    ZTPMetadata, ZTPRequest, ZTPRequestCode, ZTPResponse, ZTPResponseCode, ZTPResponseData
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
        
        send_request(&socket);
        println!("Sent GET request to {SERVER_ADDRESS}");

        let metadata = receive_metadata(&socket);
        if metadata.is_none() {
            println!("Connection Timeout: Metadata did not arrive");
            return;
        }
        dbg!(&metadata);

        let resource = receive_resource(&socket, metadata.unwrap());
        if resource.is_none() {
            println!("Connection Timeout: Resource did not arrive");
            return;
        }
        let save_path = format!("{CLIENT_DIR_PATH}/{RES_NAME}");
        
        println!("Saving Resource to: {save_path}");
        fs::write(&save_path, &resource.unwrap()).unwrap();
    }
}

fn send_request(socket: &UdpSocket){
    let req = ZTPRequest::new(ZTPRequestCode::Get, RES_NAME.to_string());
    let bytes = ZTPRequest::encode_to_vec(req);
    socket.send(&bytes).unwrap();
}

fn receive_metadata(socket: &UdpSocket) -> Option<ZTPMetadata>{
    let mut rx_buff = [0u8; 4096];
    let mut tx_buff = [0u8; 4096];
    let mut tries = 0;

    while tries <= MAX_RETRIES{
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;
        if let Err(_) = socket.peek(&mut rx_buff){
            continue;
        }
        
        let bytes = socket.recv(&mut rx_buff).unwrap();
        if let Some(res) =  parse_response(&rx_buff[..bytes]){
            send_ack(socket, &mut tx_buff);
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
    let mut received_pkgs: HashSet<u64> = HashSet::with_capacity(metadata.count());
    let mut rng = rand::rng();

    println!("Receiving resource");
    while res_code != ZTPResponseCode::EndRequest{
    println!("Try {tries}");
        thread::sleep(Duration::from_millis(TTL_MILLIS));
        tries += 1;

        if let Err(_) = socket.peek(&mut rx_buff){
            if tries > MAX_RETRIES { return None}
            continue;
        }
        tries = 0;

        let bytes = socket.recv(&mut rx_buff).unwrap();
        
        if let Some(response) = parse_response(&rx_buff[..bytes]){
            process_response(
                response, 
                &mut res_buff, 
                socket, 
                &mut tx_buff,
                &mut res_code,
                &mut received_pkgs,
                &mut rng,
            ); 
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
    if let Ok(res) = ZTPResponse::decode_from_slice(buffer){
        return Some(res.0);
    }
    None
}

fn process_response(
    response: ZTPResponse, 
    res_buff: &mut Vec<u8>,
    socket: &UdpSocket,
    tx_buff: &mut [u8],
    res_code: &mut ZTPResponseCode,
    received_pkgs: &mut HashSet<u64>,
    rng: &mut ThreadRng, 
){
    *res_code = response.get_code();
    if *res_code == ZTPResponseCode::Data{
        let data = response.get_bytes().unwrap();
        let hash_result = calculate_hash(data, rng);
        let incoming_hash = response.get_hash().unwrap();
        let pkg_id = response.get_pkg_id().unwrap();
        println!("Incoming Hash: {incoming_hash}; Calculated Hash: {hash_result}");
        if hash_result == incoming_hash && !received_pkgs.contains(&pkg_id){
            copy_data(res_buff, data);
            received_pkgs.insert(pkg_id);
            println!("Received {} bytes from {SERVER_ADDRESS}", data.len());
            println!("Total Received: {}", res_buff.len());
            send_ack(socket, tx_buff);
        }
        else{
            send_nack(socket, tx_buff);
        }
    }
}

fn send_ack(socket: &UdpSocket, tx_buff: &mut [u8]) -> usize{
    let ack = ZTPResponse::new(
        ZTPResponseCode::Ack,
        None,
        None
    );
    println!("Sending ACK");
    let bytes = ZTPResponse::encode_into_slice(ack, tx_buff).unwrap();
    socket.send(&tx_buff[..bytes]).unwrap()
}


fn send_nack(socket: &UdpSocket, tx_buff: &mut [u8]) -> usize{
    println!("Sending ACK");
    let nack = ZTPResponse::new(
        ZTPResponseCode::Nack,
        None,
        None
    );
    let bytes = ZTPResponse::encode_into_slice(nack, tx_buff).unwrap();
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

fn calculate_hash(data: &[u8], rng: &mut ThreadRng) -> u64{
    let rand_number = rng.random_range(0u8..100);
    let hash_result = xxh3::xxh3_64(data);
    if rand_number  < ERROR_CHANCE{
        return 0; 
    }
    hash_result
}
