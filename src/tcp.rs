use log::{error, info};
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::cipher::{decrypt_host, xor_cipher};
use crate::dns::dns_tcp_over_udp;

const HOST_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"Meng:\s*(.*)\r").unwrap());

pub fn get_proxy_host(buf: &[u8]) -> Option<String> {
    for cap in HOST_RE.captures_iter(&buf) {
        return match cap.get(1) {
            None => None,
            Some(host) => Some(String::from_utf8_lossy(host.as_bytes()).to_string()),
        };
    }
    return None;
}

pub async fn tcp_forward(src: &mut ReadHalf<'_>, dest: &mut WriteHalf<'_>) {
    let mut buf = [0; 65536];
    let mut rem: usize = 0;
    while let Ok(len) = src.read(&mut buf).await {
        if len > 0 {
            rem = xor_cipher(&mut buf[..len], "quanyec", rem);
            if let Err(err) = dest.write(&mut buf[..len]).await {
                error!("Write data occurred error: {}", err.to_string());
                break;
            }
        } else {
            // end of file
            break;
        }
    }
}

pub async fn handle_tcp_session(mut socket: &mut TcpStream, mut buf: &mut [u8]) {
    let mut host = match get_proxy_host(&buf) {
        Some(host) => host,
        None => return,
    };
    let mut host = match decrypt_host(&mut host) {
        Some(host) => host,
        None => return,
    };
    info!("proxy host: {}", host);

    // TODO: `dns-over-udp` configuration
    if host.ends_with(":53") {
        dns_tcp_over_udp(&mut socket, &host, &mut buf).await;
        return;
    }

    if !host.contains(":") {
        host.push_str(":80")
    }

    let mut dest = match TcpStream::connect(&host).await {
        Ok(socket) => socket,
        Err(err) => {
            error!("Connect to {} failed, reason: {}", host, err.to_string());
            return;
        }
    };
    let (mut sread, mut swrite) = socket.split();
    let (mut dread, mut dwrite) = dest.split();
    tokio::join!(
        tcp_forward(&mut dread, &mut swrite),
        tcp_forward(&mut sread, &mut dwrite),
    );
    info!("tcp connection ended: {}", host);
}

#[test]
fn get_proxy_host_test() {
    let buf = b"Meng:   m.quanye.org\r\nla";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
    let buf = b"abcMeng:   m.quanye.org\r\nla";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
    let buf = b"abcMeng:   m.quanye.org\r\n\r\n";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
}
