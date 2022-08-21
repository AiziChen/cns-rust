use crate::cipher::xor_cipher;
use log::{error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UdpSocket},
};

pub async fn dns_tcp_over_udp(socket: &mut TcpStream, host: &str, mut buf: &mut [u8]) {
    info!("Starting dns-tcp-over-udp");

    let rlen = match socket.read(&mut buf).await {
        Ok(len) if len == 0 => return,
        Ok(len) => len,
        Err(err) => {
            error!(
                "dns-tcp-over-udp: could not read data from buf: {}",
                err.to_string()
            );
            return;
        }
    };
    xor_cipher(&mut buf[..rlen], "quanyec", 0);

    let udp_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    match udp_socket.send_to(&buf[2..rlen], &host).await {
        Ok(len) => len,
        Err(err) => {
            error!(
                "connected to host {} occurred error: {}",
                &host,
                err.to_string()
            );
            socket
                .write_all(format!("Proxy address [{}] DNS Dial() error", &host).as_bytes())
                .await
                .unwrap();
            return;
        }
    };
    let rlen = match udp_socket.recv(&mut buf[2..]).await {
        Ok(len) if len <= 0 => return,
        Ok(len) => len,
        Err(err) => {
            error!(
                "receive message from host {} occurred error: {}",
                &host,
                err.to_string()
            );
            return;
        }
    };
    buf[0] = (rlen >> 8) as u8;
    buf[1] = rlen as u8;
    xor_cipher(&mut buf[..2 + rlen], "quanyec", 0);
    socket.write_all(&buf[..2 + rlen]).await.unwrap();
}
