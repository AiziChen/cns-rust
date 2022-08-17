use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::cipher::{decrypt_host, xor_cipher};

pub fn get_proxy_host(buf: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buf);
    let re = Regex::new(r"Meng:\s*(.*)\r").unwrap();
    for cap in re.captures_iter(s.as_ref()) {
        return match cap.get(1) {
            None => None,
            Some(host) => Some(String::from(host.as_str())),
        };
    }
    return None;
}

pub async fn tcp_forward(src: &mut OwnedReadHalf, dest: &mut OwnedWriteHalf) {
    let mut buf = [0; 65536];
    let mut rem: usize = 0;
    while let Ok(len) = src.read(&mut buf).await {
        rem = xor_cipher(&mut buf[..len], "quanyec", rem);
        dest.write(&mut buf[..len]).await.unwrap();
    }
}

pub async fn handle_tcp_session(mut socket: TcpStream, buf: &[u8]) {
    let mut host = match get_proxy_host(buf) {
        Some(host) => host,
        None => return,
    };
    let mut host = match decrypt_host(&mut host) {
        Some(host) => host,
        None => return,
    };
    println!("proxy host: {}", host);

    if !host.contains(":") {
        host.push_str(":80")
    }

    let dest = TcpStream::connect(host).await.unwrap();
    let (mut sread, mut swrite) = socket.into_split();
    let (mut dread, mut dwrite) = dest.into_split();
    tokio::join!(
        tcp_forward(&mut dread, &mut swrite),
        tcp_forward(&mut sread, &mut dwrite),
    );
    println!("connection has ended.");
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
