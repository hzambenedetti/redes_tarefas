use std::{
    collections::HashMap, env
};

use application::{
    server::Server,
    client::Client,
};

pub mod application;
pub mod constants;

fn main() {

    let var_map = collect_vars();
    if let Some(role) = var_map.get("role"){
        match role.as_str(){
            "server" => 
            {
                let mut server = Server::new();
                server.run()
            },
            _ => {
                let mut client = Client::new();
                client.run()
            }
        }
    }
    else{
       let mut client = Client::new();
       client.run(); 
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
