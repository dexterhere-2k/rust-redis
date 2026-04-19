#![allow(unused_imports)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let data: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let data_lock = Arc::clone(&data);
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
                                        "SET" => {
                                            let key = parts[4].to_string();
                                            let value = parts[6].to_string();
                                            data_lock.lock().unwrap().insert(key, value);
                                            let response = "+OK\r\n";
                                            stream.write_all(response.as_bytes()).await.is_err();
                                        }
                                        "GET" => {
                                            if parts.len() >= 5 {
                                                let key = parts[4];
                                                // let data_lock = Arc::clone(&data);
                                                let response = {
                                                    let val = data_lock.lock().unwrap(); // Lock acquired
                                                    if let Some(value) = val.get(key) {
                                                        format!("${}\r\n{}\r\n", value.len(), value)
                                                    } else {
                                                        String::from("$-1\r\n")
                                                    }
                                                };
                                                let _ = stream.write_all(response.as_bytes()).await;
                                            }
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
