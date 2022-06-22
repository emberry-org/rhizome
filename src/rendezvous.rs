use std::{collections::HashMap, io, net::SocketAddr};

use tokio::net::UdpSocket;

type MatchMap = HashMap<[u8; 64], SocketAddr>;

pub async fn handle(
    socket: &UdpSocket,
    matchmap: &mut MatchMap,
    packet: &[u8; 64],
    addr: SocketAddr,
) -> io::Result<()> {
    if let Some(other) = matchmap.remove(packet) {
        println!("Matching {} and {}", addr, other);

        make_match(socket, addr, other).await
    } else {
        println!("{} wants room at {}", addr, String::from_utf8_lossy(packet));
        
        matchmap.insert(*packet, addr);
        Ok(())
    }
}

/// Sends `addr1`'s IP address and external port to `addr2` using `socket` and vice versa.
/// # Errors
/// Returns any error produced by `socket.send_to(...)`, or an
/// [`InvalidInput`](https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.InvalidInput)
/// if one of the supplied addresses is ipv4 and the other is ipv6
pub async fn make_match(
    socket: &UdpSocket,
    addr1: SocketAddr,
    addr2: SocketAddr,
) -> io::Result<()> {
    if addr1.is_ipv4() != addr2.is_ipv4() {
        socket.send_to(&[0u8], addr1).await?;
        socket.send_to(&[0u8], addr2).await?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "addr1 and addr2 be of the same type (ipv4, ipv6)",
        ));
    } else {
        send_info(socket, addr1, addr2).await?;
        send_info(socket, addr2, addr1).await?;
    }
    Ok(())
}

async fn send_info(socket: &UdpSocket, to: SocketAddr, about: SocketAddr) -> Result<(), io::Error> {
    let mut ip_raw = match about {
        SocketAddr::V4(s) => s.ip().octets().to_vec(),
        SocketAddr::V6(s) => s.ip().octets().to_vec(),
    };
    let mut packet = vec![];
    packet.push(if about.is_ipv4() { 4u8 } else { 6u8 });
    packet.append(&mut ip_raw);
    packet.push((about.port() >> 8) as u8);
    packet.push((about.port() % 0x100) as u8);

    socket.send_to(&packet, to).await?;
    Ok(())
}
