use std::io::Error;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::cipher::{decrypt_host, xor_cipher};
use crate::dns::dns_tcp_over_udp;
use crate::tools::get_proxy_host;

pub async fn tcp_forward(src: &mut ReadHalf<'_>, dest: &mut WriteHalf<'_>) -> Result<(), Error> {
    let mut buf = [0u8; 65536];
    let mut rem: usize = 0;
    loop {
        match src.read(&mut buf).await {
            Ok(len) => {
                if len > 0 {
                    rem = xor_cipher(&mut buf[..len], "quanyec", rem);
                    if let Err(err) = dest.write(&buf[..len]).await {
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

pub async fn handle_tcp_session(stream: &mut TcpStream, header: &str) {
    let Some(host) = get_proxy_host(header) else {
        return;
    };
    let Some(mut host) = decrypt_host(&host) else {
        return;
    };
    info!("proxy host: {}", host);

    // TODO: `dns-over-udp` configuration
    // if host.ends_with(":53") {
    //     dns_tcp_over_udp(stream, &host, header).await;
    //     return;
    // }

    if !host.contains(':') {
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
    // let mut forward1 = tokio::spawn(async move {
    //     if tcp_forward(&mut dread, &mut swrite).await.is_err() {
    //         return;
    //     }
    // });
    // let mut forward2 = tokio::spawn(async move {
    //     if tcp_forward(&mut sread, &mut dwrite).await.is_err() {
    //         return;
    //     }
    // });
    // tokio::select! {
    //     _ = &mut forward1 => forward2.abort(),
    //     _ = &mut forward2 => forward1.abort(),
    // }
    tokio::try_join!(
        tcp_forward(&mut dread, &mut swrite),
        tcp_forward(&mut sread, &mut dwrite),
    );
    info!("tcp connection ended: {}", host);
}

#[test]
fn get_proxy_host_test() {
    let buf = "Meng:   m.quanye.org\r\nla";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
    let buf = "abcMeng:   m.quanye.org\r\nla";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
    let buf = "abcMeng:   m.quanye.org\r\n\r\n";
    assert!(get_proxy_host(buf).unwrap().eq("m.quanye.org"));
}
