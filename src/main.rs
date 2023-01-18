use std::error::Error;

use async_recursion::async_recursion;
use config::set_max_nofile;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

use crate::config::enable_tcp_fastopen;
use crate::tcp::handle_tcp_session;
use crate::tools::{bytes_contains, is_http_header};
use crate::udp::handle_udp_session;

mod cipher;
mod config;
mod dns;
mod tcp;
mod tools;
mod udp;

async fn response_header(stream: &mut TcpStream, buf: &[u8]) -> bool {
    if bytes_contains(buf, "WebSocket".as_bytes()) {
        if let Err(e) = stream.write_all("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: CuteBi Network Tunnel, (%>w<%)\r\n\r\n".as_bytes()).await {
            error!("failed to write to socket; err = {}", e.to_string());
            return false;
        }
    } else if bytes_contains(buf, "CON".as_bytes()) {
        if let Err(e) = stream.write_all("HTTP/1.1 200 Connection established\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n".as_bytes()).await {
            error!("failed to write to socket; err = {}", e.to_string());
            return false;
        }
    } else if let Err(e) = stream.write_all("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n".as_bytes()).await {
        error!("failed to write to socket; err = {}", e.to_string());
        return false;
    }

    true
}

#[async_recursion]
async fn handle_connection(mut stream: &mut TcpStream) {
    let mut buf = [0; 65536];
    let len = match (&mut stream).read(&mut buf).await {
        Ok(len) if len == 0 => return,
        Ok(len) => len,
        Err(err) => {
            error!("failed to read from socket; reason = {}", err.to_string());
            return;
        }
    };

    if is_http_header(&buf[..len]) {
        /* process TCP */
        let status = response_header(stream, &buf[..len]).await;
        if status {
            if !bytes_contains(&buf[..len], b"httpUDP") {
                handle_tcp_session(stream, &mut buf[..len]).await;
            } else {
                handle_connection(stream).await;
            }
        }
    } else {
        /* process UDP */
        handle_udp_session(stream, &mut buf[..len]).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "cns_rust");
    env_logger::init();
    set_max_nofile();
    let listener = TcpListener::bind("[::]:1080").await?;
    enable_tcp_fastopen(&listener);
    loop {
        let (mut stream, _) = listener.accept().await?;
        spawn(async move {
            info!("Handle a new connection...");
            handle_connection(&mut stream).await;
        });
    }
}
