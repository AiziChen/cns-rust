use std::io::Error;

use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::cipher::{decrypt_host, xor_cipher};
use crate::dns::dns_tcp_over_udp;
use crate::tools::get_proxy_host;

pub async fn tcp_forward(src: &mut ReadHalf<'_>, dest: &mut WriteHalf<'_>) -> Result<(), Error> {
    let mut buf = [0; 65536];
    let mut rem: usize = 0;
    loop {
        match src.read(&mut buf).await {
            Ok(len) => {
                if len > 0 {
                    rem = xor_cipher(&mut buf[..len], "quanyec", rem);
                    if let Err(err) = dest.write(&mut buf[..len]).await {
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

pub async fn handle_tcp_session(mut stream: &mut TcpStream, mut buf: &mut [u8]) {
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
        dns_tcp_over_udp(&mut stream, &host, &mut buf).await;
        return;
    }

    if !host.contains(":") {
        host.push_str(":80")
    }

    let mut dest = match TcpStream::connect(&host).await {
        Ok(stream) => stream,
        Err(err) => {
            error!("Connect to {} failed, reason: {}", host, err.to_string());
            return;
        }
    };
    let (mut sread, mut swrite) = stream.split();
    let (mut dread, mut dwrite) = dest.split();

    info!("starting tcp forward...");
    let _ = tokio::try_join!(
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
