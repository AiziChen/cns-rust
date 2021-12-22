use regex::Regex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};

use crate::cipher::{decrypt_host};

pub fn get_proxy_host(buf: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buf);
    let re = Regex::new("Meng:\\s*(.*)\r").unwrap();
    for n in re.captures_iter(s.as_ref()) {
        return match n.get(1) {
            None => None,
            Some(host) => unsafe {
                let mut host = String::from(host.as_str()).to_string();
                decrypt_host(&mut host)
            },
        };
    }
    return None;
}

pub async fn tcp_forward(src: &mut TcpStream, dest: &mut TcpStream) {
    let mut d_buf = [0; 65535];
    let mut len = src.read(&mut d_buf).await.unwrap();
    while len > 0 {
        len = dest.write(&d_buf[0..len]).await.unwrap();
        if len != 65535 {
            break;
        } else {
            len = src.read(&mut d_buf).await.unwrap();
        }
    }
}

pub async fn handle_tcp_session(socket: &mut TcpStream, buf: &[u8]) {
    let mut host = match get_proxy_host(buf) {
        Some(host) => host,
        None => return,
    };
    println!("proxy host: {}", host);

    if !host.contains(":") {
        host.push_str(":80")
    }

    let mut d_stream = TcpStream::connect(host).await.unwrap();
    // spawn(async move {
    tcp_forward(socket, &mut d_stream).await;
    // });
    tcp_forward(&mut d_stream, socket).await;
}

#[test]
fn get_proxy_host_test() {
    let buf = b"abcMeng:   m.quanye.org\r\nla";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
}
