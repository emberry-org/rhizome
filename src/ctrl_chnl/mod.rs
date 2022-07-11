mod state;

use self::state::State;
use crate::server::messages::{self, RoomProposal, ServerMessage, SocketMessage};
use smoke::messages as smokemsg;
use smoke::messages::{EmbMessage, RhizMessage};
use smoke::User;
use std::io;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

/// Handle an incoming connection.
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
    let state = State {
        user,
        com,
        tx: tx.clone(),
    };
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

    // write result in status and avoid returning from the function this way
    let status = handle_messages(&state, &mut tls, &mut rx).await;

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

/// Handle any incoming messages over the TLS.
async fn handle_messages<T>(
    state: &State,
    tls: &mut BufReader<TlsStream<T>>,
    rx: &mut Receiver<ServerMessage>,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut recv_buf = Vec::with_capacity(smokemsg::EMB_MESSAGE_BUF_SIZE);

    // Repeatedly handle requests
    loop {
        select! {
            req = EmbMessage::recv_req(tls, &mut recv_buf) => {
                match req? {
                    EmbMessage::Heartbeat => continue,
                    EmbMessage::Shutdown => return Ok(()),
                    EmbMessage::Room(user) => handle_room_request(state, user, tls).await?,
                    EmbMessage::Accept(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "cannot handle accept as request"))
                }
            }
            Some(msg) = rx.recv() => {
                match msg {
                    ServerMessage::RoomProposal { proposal } => handle_room_proposal(state, proposal, tls, &mut recv_buf).await?,
                    ServerMessage::RoomAffirmation { room_id } => RhizMessage::AcceptedRoom(room_id).send_with(tls).await?,
                }
            }
        }
    }
}

async fn handle_room_proposal<T>(
    state: &State,
    proposal: messages::RoomProposal,
    tls: &mut BufReader<TlsStream<T>>,
    recv_buf: &mut smokemsg::EmbMessageBuf,
) -> io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    if let Err(err) = RhizMessage::WantsRoom(proposal.proposer)
        .send_with(tls)
        .await
    {
        proposal
            .proposer_tx
            .send(ServerMessage::RoomAffirmation { room_id: None })
            .await
            .unwrap_or(());
        return Err(err);
    }

    let resp = EmbMessage::recv_req(tls, recv_buf).await?;

    match resp {
        EmbMessage::Accept(true) => state
            .com
            .send(SocketMessage::GenerateRoom {
                proposer: proposal.proposer_tx,
                recipient: state.tx.clone(),
            })
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "Internal communication channel broken",
                )
            }),
        EmbMessage::Accept(false) => {
            proposal
                .proposer_tx
                .send(ServerMessage::RoomAffirmation { room_id: None })
                .await
                .unwrap_or(());
            Ok(())
        }
        _ => {
            proposal
                .proposer_tx
                .send(ServerMessage::RoomAffirmation { room_id: None })
                .await
                .unwrap_or(());
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot handle non accept as response to request",
            ))
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
        None => RhizMessage::NoRoute(user).send_with(tls).await,
        Some(route) => {
            match route
                .send(ServerMessage::RoomProposal {
                    proposal: RoomProposal {
                        proposer: state.user,
                        proposer_tx: state.tx.clone(),
                    },
                })
                .await
            {
                Ok(()) => RhizMessage::HasRoute(user).send_with(tls).await,
                // send no route if the receiver of the peer has been dropped by now (peer disconnected)
                Err(_) => RhizMessage::NoRoute(user).send_with(tls).await,
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
