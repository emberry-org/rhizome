mod request;
mod response;

use crate::ctrl_chnl::response::Response;
use crate::server::messages::{ServerMessage, SocketMessage};
use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::server::user::User;

use self::request::Request;

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

    handle_messges(&mut tls, &mut rx, &com).await?;

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

async fn handle_messges<T>(
    tls: &mut BufReader<TlsStream<T>>,
    rx: &mut Receiver<ServerMessage>,
    com: &Sender<SocketMessage>,
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
                    Request::RoomRequest(user) => handle_room_request(user, tls, com).await?,
                }
            }
            Some(msg) = rx.recv() => {
                // handle messages from other server parts trying to speak with you
                todo!();
            }
        }
    }
}

async fn handle_room_request<T>(
    user: User,
    tls: &mut BufReader<TlsStream<T>>,
    com: &Sender<SocketMessage>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let (tx, rx) = oneshot::channel();
    com.send(SocketMessage::RoomRequest {
        receiver: user,
        success: tx,
    })
    .await
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "Internal communication channel broken",
        )
    })?;

    let route = rx.await.map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "Internal communication channel broken",
        )
    })?;

    match route {
        None => Response::NoRoute(user).send_with(tls).await,
        Some(route) => {
            todo!("communicate room proposal")
        },
    }
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
    // read the tls strea for <size> bytes and parse request
    todo!()
}
