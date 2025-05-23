#![allow(unused_imports)]
use std::{io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9092").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                let header = ResponseHeaderV0 {
                    message_size: 0,
                    correlation_id: 7,
                };
                let encoded_header = header.encode();
                _stream.write_all(&encoded_header).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

struct ResponseHeaderV0 {
    message_size: i32,
    correlation_id: i32,
}

impl ResponseHeaderV0 {
    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.message_size.to_be_bytes());
        buffer.extend_from_slice(&self.correlation_id.to_be_bytes());
        buffer
    }
}
