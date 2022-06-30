use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::user::User;

pub async fn handle(socket: (TcpStream, SocketAddr), acceptor: TlsAcceptor) -> io::Result<()> {
    let mut tls = rhizome_handshake(socket.0, &socket.1, acceptor).await?;
    
    let _user = timeout(Duration::from_secs(10), authenticate(&mut tls)).await??;
    
    Ok(())
}

async fn rhizome_handshake<T>(
    stream: T,
    addr: &SocketAddr,
    acceptor: TlsAcceptor,
) -> io::Result<BufReader<TlsStream<T>>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // perform TLS handshake
    let mut tls = match acceptor.accept(stream).await {
        Ok(tls) => {
            println!("tls established: {}", addr);
            tls
        }
        Err(e) => {
            eprintln!("tls handshake failed at {} with {}", addr, e);
            return Err(e);
        }
    };

    // Send rhizome hello
    tls.write_all(concat!("rhizome v", env!("CARGO_PKG_VERSION"), '\n').as_bytes())
        .await?;

    Ok(BufReader::new(tls))
}

async fn authenticate<T>(tls: &mut BufReader<TlsStream<T>>) -> io::Result<User>
where T: AsyncRead + AsyncWrite + Unpin
{
    let mut buf = [0u8;32];
    tls.read_exact(&mut buf).await?;

    Ok(User::Authenticated(buf))
}