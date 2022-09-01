use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::cipher::xor_cipher;
use log::{error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream, UdpSocket,
    },
};

async fn udp_server_to_client(socket: &UdpSocket, cwrite: &mut WriteHalf<'_>, mut s2c_rem: usize) {
    let mut ignore_head_len: usize;
    let mut payload = [0u8; 65536];
    loop {
        let (payload_len, addr) = match socket.recv_from(&mut payload[24..]).await {
            Ok((len, _)) if len == 0 => break,
            Ok(rs) => rs,
            Err(err) => {
                error!(
                    "receive data from addr {} failed, reason: {}",
                    socket.peer_addr().unwrap().to_string(),
                    err.to_string()
                );
                break;
            }
        };
        info!(
            "read udp server len: {}, raddr: {}",
            payload_len,
            addr.to_string()
        );
        if addr.is_ipv4() {
            ignore_head_len = 12;
            payload[12] = (payload_len + 10) as u8;
            payload[13] = ((payload_len + 10) >> 8) as u8;
            payload[14..18].copy_from_slice(&[0, 0, 0, 1]);
            if let IpAddr::V4(ip) = addr.ip() {
                payload[18..22].copy_from_slice(&ip.octets());
            } else {
                error!("failed to get v4 address {}", addr.to_string());
                break;
            }
        } else {
            ignore_head_len = 0;
            payload[0] = (payload_len + 22) as u8;
            payload[1] = ((payload_len + 22) >> 8) as u8;
            payload[2..6].copy_from_slice(&[0, 0, 0, 3]);
            if let IpAddr::V6(ip) = addr.ip() {
                payload[6..22].copy_from_slice(&ip.octets());
            } else {
                error!("failed to get v6 address {}", addr.to_string());
                break;
            }
        }

        payload[22] = (addr.port() >> 8) as u8;
        payload[23] = addr.port() as u8;
        s2c_rem = xor_cipher(
            &mut payload[ignore_head_len..24 + payload_len],
            "quanyec",
            s2c_rem,
        );
        if let Err(err) = cwrite
            .write(&payload[ignore_head_len..24 + payload_len])
            .await
        {
            error!("Write udp data to server error: {}", err.to_string());
            break;
        }
    }
}

async fn write_to_server(socket: &UdpSocket, buf: &mut [u8]) -> i32 {
    let mut pkg_sub = 0usize;
    while pkg_sub + 2 < buf.len() {
        let pkg_len = (buf[pkg_sub] as u16 | ((buf[pkg_sub + 1] as u16) << 8)) as usize;
        info!("pkgSub: {}, pkgLen: {}, {}", pkg_sub, pkg_len, buf.len());
        if pkg_sub + 2 + pkg_len > buf.len() || pkg_len <= 10 {
            return 0;
        }
        if buf.starts_with(&[0u8; 2]) {
            return 1;
        }
        let (addr, udp_header_len) = if buf[5] == 1 {
            /* ipv4 */
            let ipv4 = Ipv4Addr::from(u32::from_be_bytes(
                buf[pkg_sub + 6..pkg_sub + 10].try_into().unwrap(),
            ));
            let addr = SocketAddr::new(
                IpAddr::V4(ipv4),
                ((buf[pkg_sub + 10] as u16) << 8) | (buf[pkg_sub + 11] as u16),
            );
            (addr, 12)
        } else {
            if pkg_len <= 24 {
                return 0;
            }
            /* ipv6 */
            let ipv6 = Ipv6Addr::from(u128::from_be_bytes(
                buf[pkg_sub + 6..pkg_sub + 22].try_into().unwrap(),
            ));
            let addr = SocketAddr::new(
                IpAddr::V6(ipv6),
                ((buf[pkg_sub + 22] as u16) << 8) | buf[pkg_sub + 23] as u16,
            );
            (addr, 24)
        };
        // write to destination
        if let Err(err) = socket
            .send_to(
                &buf[(pkg_sub + udp_header_len)..(pkg_sub + 2 + pkg_len)],
                addr,
            )
            .await
        {
            error!("send client data to UDP server error: {}", err.to_string());
            return -1;
        }

        pkg_sub = pkg_sub + 2 + pkg_len;
    }

    return pkg_sub as i32;
}

async fn udp_client_to_server(
    udp_socket: &UdpSocket,
    cread: &mut ReadHalf<'_>,
    mut buf: &mut [u8],
    mut c2s_rem: usize,
) {
    let wlen = write_to_server(udp_socket, &mut buf).await;
    if wlen == -1 {
        return;
    }
    let mut payload = [0u8; 65536];
    let mut payload_len: usize;
    let wlen = wlen as usize;
    if wlen < buf.len() {
        payload_len = buf.len() - wlen;
        payload[..payload_len].copy_from_slice(&buf[wlen..]);
    } else {
        payload_len = 0;
    };
    loop {
        let rlen = match cread.read(&mut buf).await {
            Ok(len) if len == 0 => break,
            Ok(len) => len,
            Err(err) => {
                error!("read data occurred error from client: {}", err.to_string());
                break;
            }
        };
        c2s_rem = xor_cipher(
            &mut payload[payload_len..payload_len + rlen],
            "quanyec",
            c2s_rem,
        );
        payload_len += rlen;
        let wlen = write_to_server(udp_socket, &mut payload[..payload_len]).await;
        if wlen == -1 {
            break;
        }
        let wlen = wlen as usize;
        if wlen < payload_len {
            payload.copy_within(wlen..payload_len, 0);
            payload_len = payload_len - wlen;
        } else {
            payload_len = 0;
        }
        info!("payload_len: {}, rlen: {}", payload_len, rlen);
    }
}

pub async fn handle_udp_session(cstream: &mut TcpStream, mut buf: &mut [u8]) {
    let mut de = [0u8; 5];
    de.copy_from_slice(&buf[..5]);
    xor_cipher(&mut de, "quanyec", 0);

    let c2s_rem = if de[2] == 0 || de[3] == 0 || de[4] == 0 {
        xor_cipher(&mut buf, "quanyec", 0)
    } else {
        error!("Not httpUDP protocol");
        return;
    };

    let udp_socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(socket) => socket,
        Err(err) => {
            error!("Bind udp error: {}", err.to_string());
            return;
        }
    };

    info!("starting UDP forward...");
    let (mut cread, mut cwrite) = cstream.split();
    tokio::join!(
        udp_client_to_server(&udp_socket, &mut cread, buf, c2s_rem),
        udp_server_to_client(&udp_socket, &mut cwrite, 0)
    );
}
