mod request;
mod response;
mod state;

use crate::ctrl_chnl::response::Response;
use crate::server::messages::{self, RoomProposal, ServerMessage, SocketMessage};
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
use self::state::State;

pub async fn handle(
    socket: (TcpStream, SocketAddr),
    acceptor: TlsAcceptor,
    com: Sender<SocketMessage>,
) -> io::Result<()> {
    let mut tls = rhizome_handshake(socket.0, &socket.1, acceptor).await?;

    // Authenticate the user but only give crate::TIMEOUT time for this operation
    let user = timeout(crate::TIMEOUT, authenticate(&mut tls)).await??;

    // Create channel for this user and push it to the main server map
    let (tx, mut rx) = mpsc::channel(100);
    // Generate state and store the authenticated user
    let state = State { user, com, tx: tx.clone()};
    state
        .com
        .send(SocketMessage::SubscribeUser { user, tx })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Internal communication channel broken",
            )
        })?;

    // [!] From here on ensure the the function does not return before disconnect is not signaled to the server [!]

    // write result in status and avoid returning from the function this way
    let status = handle_messges(&state, &mut tls, &mut rx).await;

    // Remove the client from the map in case of disconnect
    state
        .com
        .send(SocketMessage::Disconnect { user })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Internal communication channel broken",
            )
        })?;

    // Return the status to get status indication
    status
}

async fn handle_messges<T>(
    state: &State,
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
                    Request::RoomRequest(user) => handle_room_request(state, user, tls).await?,
                }
            }
            Some(msg) = rx.recv() => {
            }
        }
    }
}

async fn handle_room_request<T>(
    state: &State,
    user: User,
    tls: &mut BufReader<TlsStream<T>>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // create response channel and request the route to the peer user
    let (tx, rx) = oneshot::channel();
    state
        .com
        .send(SocketMessage::RoomRequest {
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

    // receive an option to a peer user (None if the other user is not connected)
    let route = rx.await.map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "Internal communication channel broken",
        )
    })?;

    match route {
        None => Response::NoRoute(user).send_with(tls).await,
        Some(route) => {
            match route
                .send(ServerMessage::RoomProposal {
                    proposal: RoomProposal {
                        proposer: state.user,
                        proposal: None,
                        proposer_tx: state.tx.clone(),
                    },
                })
                .await
            {
                Ok(()) => Response::HasRoute(user).send_with(tls).await,
                // send no route if the receiver of the peer has been dropped by now (peer disconnected)
                Err(_) => Response::NoRoute(user).send_with(tls).await,
            }
        }
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
