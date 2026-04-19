#![allow(unused_imports)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let data: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>> =
        Arc::new(Mutex::new(HashMap::new()));
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
                                            let _ = stream.write_all(response.as_bytes()).await;
                                        }
                                        "ECHO" => {
                                            let arg = parts[4];
                                            let response = format!("${}\r\n{}\r\n", arg.len(), arg);
                                            let _ = stream.write_all(response.as_bytes()).await;
                                        }
                                        "SET" => {
                                            let key = parts[4].to_string();
                                            let value = parts[6].to_string();
                                            let mut ttl: Option<Instant> = None;

                                            if parts.len() >= 11 {
                                                let cmd = parts[8].to_uppercase();
                                                if let Ok(time_val) = parts[10].parse::<u64>() {
                                                    if cmd == "PX" {
                                                        ttl = Some(
                                                            Instant::now()
                                                                + Duration::from_millis(time_val),
                                                        );
                                                    } else if cmd == "EX" {
                                                        ttl = Some(
                                                            Instant::now()
                                                                + Duration::from_secs(time_val),
                                                        );
                                                    }
                                                }
                                            }
                                            data_lock.lock().unwrap().insert(key, (value, ttl));
                                            let response = "+OK\r\n";
                                            let _ = stream.write_all(response.as_bytes()).await;
                                        }
                                        "GET" => {
                                            if parts.len() >= 5 {
                                                let key = parts[4];

                                                let response = {
                                                    let mut val = data_lock.lock().unwrap();
                                                    if let Some((value, ttl)) = val.get(key) {
                                                        if let Some(exp_time) = ttl {
                                                            if Instant::now() > *exp_time {
                                                                val.remove(key);
                                                                String::from("$-1\r\n")
                                                            } else {
                                                                format!(
                                                                    "${}\r\n{}\r\n",
                                                                    value.len(),
                                                                    value
                                                                )
                                                            }
                                                        } else {
                                                            format!(
                                                                "${}\r\n{}\r\n",
                                                                value.len(),
                                                                value
                                                            )
                                                        }
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
