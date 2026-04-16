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
            Ok((mut stream, _addr)) => loop {
                let mut buffer = [0; 512];
                match stream.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(_) => {
                        stream.write_all(b"+PONG\r\n").await.is_err();
                    }
                    Err(_) => {
                        break;
                    }
                }
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
