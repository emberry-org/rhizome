mod request;

use crate::server::messages::{ServerMessage, SocketMessage};
use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::timeout;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::server::user::User;

use self::request::Request;

/// Handle an incoming connection.
pub async fn handle(
    socket: (TcpStream, SocketAddr),
    acceptor: TlsAcceptor,
    com: Sender<SocketMessage>,
) -> io::Result<()> {
    let mut tls = rhizome_handshake(socket.0, &socket.1, acceptor).await?;

    let user = timeout(crate::TIMEOUT, authenticate(&mut tls)).await??;

    let (tx, mut rx) = mpsc::channel(100);

    com.send(SocketMessage::SubscribeUser { user, tx })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Internal communication channel broken",
            )
        })?;

    handle_messages(&mut tls, &mut rx).await?;

    com.send(SocketMessage::Disconnect { user })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Internal communication channel broken",
            )
        })?;

    Ok(())
}

/// Handle any incoming messages over the TLS.
async fn handle_messages<T>(
    tls: &mut BufReader<TlsStream<T>>,
    rx: &mut Receiver<ServerMessage>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = [0u8; 4]; // message header buffer

    // Repeatedly handle requests
    loop {
        select! {
            Ok(_) = tls.read_exact(&mut buf) => {
                let size = u32::from_be_bytes(buf);
                match timeout(crate::TIMEOUT, recv_req(tls, size)).await?? {
                    Request::Heartbeat => continue,
                    Request::Shutdown => return Ok(()),
                    Request::RoomRequest(_) => todo!(),
                }
            }
            Some(msg) = rx.recv() => {
                todo!();
            }
        }
    }
}

/// Perform a handshake and send the client a hello message.
/// 
/// Hello message : ```rhizome v<CARGO_PKG_VERSION>\n```
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

/// Authenticate a user by reading their public key from the TLS channel.
async fn authenticate<T>(tls: &mut BufReader<TlsStream<T>>) -> io::Result<User>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = [0u8; 32];
    tls.read_exact(&mut buf).await?;

    Ok(User { key: buf })
}

async fn recv_req<T>(tls: &mut BufReader<TlsStream<T>>, size: u32) -> io::Result<Request>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    todo!()
}
