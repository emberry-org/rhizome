use std::io::{self, ErrorKind, Write, BufWriter};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

pub async fn handle(socket: (TcpStream, SocketAddr), acceptor: TlsAcceptor) -> io::Result<()> {

    let mut _tls = rhizome_handshake(socket.0, &socket.1, acceptor).await?;
    println!("tls end: {}", socket.1);
    Ok(())
}

async fn rhizome_handshake<T>(stream: T, addr: &SocketAddr, acceptor: TlsAcceptor) -> io::Result<BufReader<TlsStream<T>>>
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
