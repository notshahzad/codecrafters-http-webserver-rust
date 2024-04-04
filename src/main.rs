extern crate log;
use env_logger;
use log::debug;
use log::error;
use log::info;
use log::warn;
//use log::{debug, error, info};
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::net::TcpListener;
use std::str::FromStr;

struct HttpResponse {
    response: String,
}
impl HttpResponse {
    fn new() -> Self {
        Self {
            response: String::new(),
        }
    }
    fn push_header(self: &mut Self, header: &str) {
        if !header.ends_with("\r\n") {
            let mut current_header = String::from(header);
            current_header.push_str("\r\n");
            self.response.push_str(&current_header);
        } else {
            self.response.push_str(header);
        }
    }
    fn header_ok(self: &mut Self) {
        self.response.clear();
        self.push_header("HTTP/1.1 200 OK");
    }
}

struct HttpReader<T> {
    stream: T,
}

impl<T> Read for HttpReader<T>
where
    T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buf: [u8; 4096] = [0u8; 4096];
        self.stream.read(&mut buf)
    }
}

fn main() {
    env_logger::init();

    let port = 4221;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();
    println!("listening on port {port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut http_response = HttpResponse::new();
                http_response.header_ok();
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
