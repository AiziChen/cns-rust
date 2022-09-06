use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

use log::{error, info};

use crate::cipher::{decrypt_host, xor_cipher};
use crate::dns::dns_tcp_over_udp;
use crate::tools::get_proxy_host;

pub fn tcp_forward(src: Arc<TcpStream>, dest: Arc<TcpStream>) -> Result<(), Error> {
    let mut buf = [0; 65536];
    let mut rem: usize = 0;
    loop {
        match src.as_ref().read(&mut buf) {
            Ok(len) => {
                if len > 0 {
                    rem = xor_cipher(&mut buf[..len], "quanyec", rem);
                    if let Err(err) = dest.as_ref().write(&mut buf[..len]) {
                        error!("write data occurred error: {}", err.to_string());
                        return Err(err);
                    }
                } else {
                    // end of file
                    return Ok(());
                }
            }
            Err(err) => {
                error!("read data occurred error: {}", err.to_string());
                return Err(err);
            }
        }
    }
}

pub fn handle_tcp_session(mut stream: TcpStream, mut buf: &mut [u8]) {
    let host = match get_proxy_host(&buf) {
        Some(host) => host,
        None => return,
    };
    let mut host = match decrypt_host(&host) {
        Some(host) => host,
        None => return,
    };
    info!("proxy host: {}", host);

    // TODO: `dns-over-udp` configuration
    if host.ends_with(":53") {
        dns_tcp_over_udp(&mut stream, &host, &mut buf);
        return;
    }

    if !host.contains(":") {
        host.push_str(":80")
    }

    let dest = match TcpStream::connect(&host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("Connect to {} failed, reason: {}", host, err.to_string());
            return;
        }
    };
    let stream = Arc::new(stream);
    let dest = Arc::new(dest);
    let stream1 = Arc::clone(&stream);
    let dest1 = Arc::clone(&dest);
    thread::spawn(move || {
        tcp_forward(stream1, dest1).unwrap();
    });
    tcp_forward(Arc::clone(&dest), Arc::clone(&stream)).unwrap();
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
