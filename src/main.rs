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

fn get_response_header(header: &str) -> &'static str {
    let lc_header = header.to_lowercase();
    return if lc_header.contains("websocket") {
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: CuteBi Network Tunnel, (%>w<%)\r\n\r\n"
    } else if lc_header.starts_with("connect") {
        "HTTP/1.1 200 Connection established\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n"
    } else {
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nServer: CuteBi Network Tunnel, (%>w<%)\r\nConnection: keep-alive\r\n\r\n"
    };
}

#[async_recursion]
async fn handle_connection(stream: &mut TcpStream) {
    let mut data = [0u8; 4096];
    let len = match stream.read(&mut data).await {
        Ok(len) if len == 0 => return,
        Ok(len) => len,
        Err(err) => {
            error!("failed to read from socket; reason = {}", err.to_string());
            return;
        }
    };
    let header = String::from(String::from_utf8_lossy(&data));

    if is_http_header(&header) {
        /* process TCP */
        let resp_header = get_response_header(&header);
        let wsize = stream.write_all(resp_header.as_bytes()).await.unwrap_or_else(|err| {
            error!("failed to write response header: {}", err.to_string());
            return;
        });
        if !header.contains("httpUDP") {
            handle_tcp_session(stream, &header).await;
        } else {
            handle_connection(stream).await;
        }
    } else {
        /* process UDP */
        // let mut data = [0; 4096];
        // handle_udp_session(stream, &mut data).await;
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
