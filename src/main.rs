extern crate log;
use env_logger;
mod http;
use crate::http::{HttpReader, HttpRequest};
use http::HttpResponse;
use log::{debug, error, info};
use std::{io::Write, net::TcpListener};

fn main() {
    env_logger::init();

    let port = 4221;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    println!("listening on port {port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let reader = HttpReader::new(&stream);
                let request = reader.read_request();
                if request.is_err() {
                    //info!(
                    //    "unable to read http request from stream because of {:?}",
                    //    request
                    //);
                    println!(
                        "unable to read http request from stream because of {:?}",
                        request
                    );
                }
                //rust people are going to murder me for this
                let request = request.unwrap();
                let mut http_response = HttpResponse::new();
                if request.path == "/" {
                    http_response.header_ok();
                } else {
                    http_response.push_header("HTTP/1.1 404 Not Found");
                }
                http_response.push_header("\r\n");
                match stream.write(http_response.response.as_bytes()) {
                    Ok(size) => {
                        info!("Successfully wrote {} bytes to TcpStream", size);
                    }
                    Err(e) => {
                        error!("Cant write data to TcpStream: {e:?}");
                        continue;
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
