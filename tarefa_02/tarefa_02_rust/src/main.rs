mod server;
mod client;

use crate::server::Server;
use crate::client::Client;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args{
    port: String,
    
    #[arg(short='r', long="role", default_value="client")]
    role: String
}

fn main() {
    let args = Args::parse();

    match args.role.as_str(){
        "client" => exec_client(args.port.as_str()),
        "server" => exec_server(args.port.as_str()),
        _ => print_usage(&args)
    }
}


fn exec_client(address: &str){
    let mut client = Client::new(address);
    client.run();
}

fn exec_server(address: &str){
    let mut server = Server::new(address);
    server.run();
}

fn print_usage(args: &Args){

}
