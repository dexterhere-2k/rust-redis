#![allow(unused_imports)]
// use std::{
//     io::{Read, Write},
//     net::TcpListener,
// };
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                tokio::spawn(async move {
                    loop {
                        let mut buffer = [0; 512];
                        match stream.read(&mut buffer).await {
                            Ok(0) => break,
                            Ok(n) => {
                                let request = String::from_utf8_lossy(&buffer[..n]);
                                let parts: Vec<&str> = request.split("\r\n").collect();
                                if parts.len() >= 3 {
                                    let cmd = parts[2].to_uppercase();
                                    match cmd.as_str() {
                                        "PING" => {
                                            let response = "+PONG\r\n";
                                            stream.write_all(response.as_bytes()).await.err();
                                        }
                                        "ECHO" => {
                                            let arg = parts[4];
                                            let response = format!("${}\r\n{}\r\n", arg.len(), arg);
                                            stream.write_all(response.as_bytes()).await.is_err();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
