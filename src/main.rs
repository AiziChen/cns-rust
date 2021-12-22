use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

use crate::tcp::{handle_tcp_session};
use crate::tools::{bytes_contains, is_http_header};

mod tcp;
mod tools;
mod cipher;

async fn response_header(socket: &mut TcpStream, buf: &[u8]) {
    if bytes_contains(&buf, "WebSocket".as_bytes()) {
        if let Err(e) = socket.write_all("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: CuteBi Network Tunnel, (%>w<%)\r\n\r\n".as_bytes()).await {
            eprintln!("failed to write to socket; err = {:?}", e);
        }
    } else if bytes_contains(&buf, "CON".as_bytes()) {
        if let Err(e) = socket.write_all("HTTP/1.1 200 Connection established\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n".as_bytes()).await {
            eprintln!("failed to write to socket; err = {:?}", e);
        }
    } else {
        if let Err(e) = socket.write_all("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n".as_bytes()).await {
            eprintln!("failed to write to socket; err = {:?}", e);
        }
    }
}

async fn handle_connection(socket: &mut TcpStream) {
    let mut buf = [0; 65535];
    loop {
        let len = match socket.read(&mut buf).await {
            Ok(len) if len == 0 => return,
            Ok(len) => len,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        if is_http_header(&buf[0..len]) {
            // response header
            response_header(socket, &buf[0..len]).await;
            // process tcp or udp
            if !bytes_contains(&buf[0..len], b"httpUDP") {
                handle_tcp_session(socket, &buf[0..len]).await;
            }
        } else {
            // handle_udp_session(socket);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:1080").await?;
    loop {
        let (mut socket, _) = listener.accept().await?;
        spawn(async move {
            println!("Handle a new connection...");
            handle_connection(&mut socket).await;
        });
    }
}
