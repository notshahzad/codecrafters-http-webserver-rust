extern crate log;
use env_logger;
use log::debug;
use log::error;
use log::info;
use log::warn;
//use log::{debug, error, info};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    env_logger::init();

    // Uncomment this block to pass the first stage
    let port = 4221;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    println!("listening on port {port}");

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
