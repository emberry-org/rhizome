mod certs;
mod ctrl_chnl;
mod err;
mod rendezvous;
mod server;
mod settings;
#[cfg(feature = "certgen")]
pub use certs::regenerate_certs;
use rand::{thread_rng, RngCore};
use rendezvous::RoomStatus;
use server::messages::{ServerMessage, SocketMessage};
use server::state::State;
pub use settings::Args;

use certs::{load_certs, load_private_key};
use err::eprinterr_with;
use std::time::{Duration, Instant};
use std::{path::PathBuf, sync::Arc};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::mpsc;
use tokio::{join, select};
use tokio_rustls::{rustls, TlsAcceptor};

pub const TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main(flavor = "current_thread")]
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configure the TlsAcceptor and bind the TcpListener
    let (tcp, tls) = configure_tls(args.tls_port, args.cert, args.key).await?;

    // Configure and bind the udp socket
    let udp = configue_udp(args.udp_port).await?;
    let mut state = State {
        usrs: Default::default(),
        rooms: Default::default(),
    };
    let mut udp_buff = [0u8; 64];

    let (tx, mut rx) = mpsc::channel::<SocketMessage>(100);
    // Prepare a long-running future stream to accept and serve clients.
    loop {
        let acceptor = tls.clone(); // We need a new Acceptor for each client because of TLS connection state
        select! {
            Ok(socket) = tcp.accept() => {
                tokio::spawn(eprinterr_with(ctrl_chnl::handle(socket, acceptor, tx.clone()), "control_channel")); // Spawn tokio task to handle tls control channel
            }
            Ok((size, addr)) = udp.recv_from(&mut udp_buff) => {
                if size != 64 {
                    eprintln!("udp recv {} bytes from {}", size, addr);
                    continue;
                }
                rendezvous::handle(&udp, &mut state.rooms, &udp_buff, addr).await?; // Handle rendezvous service directly since updates to matchmap should be atomic
            }
            Some(msg) = rx.recv() => {
                handle_sock_msg(msg, &mut state).await;
            }
        }
    }
}

#[inline]
async fn configue_udp(
    udp_port: u16,
) -> Result<UdpSocket, Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind(("0.0.0.0", udp_port)).await?;
    Ok(socket)
}

async fn configure_tls(
    tls_port: u16,
    cert: PathBuf,
    key: PathBuf,
) -> Result<(TcpListener, TlsAcceptor), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", tls_port);
    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs(cert)?;
        // Load private key.
        let key = load_private_key(key)?;

        let cfg = rustls::ServerConfig::builder()
            .with_safe_defaults() // Only allow safe TLS configurations
            .with_no_client_auth() // Disable client auth
            .with_single_cert(certs, key)?; // Set server certificate
        Arc::new(cfg)
    };

    // Create a TCP listener
    let tcp = TcpListener::bind(&addr).await?;
    // Create Tokio specific TlsAcceptor to handle requests
    let tls_acceptor = TlsAcceptor::from(tls_cfg);

    Ok((tcp, tls_acceptor))
}

async fn handle_sock_msg(msg: SocketMessage, state: &mut State) {
    match msg {
        SocketMessage::SubscribeUser { user, tx } => {
            state.usrs.insert(user.key, tx);
        }
        SocketMessage::Disconnect { user } => {
            state.usrs.remove(&user.key);
        }
        SocketMessage::RoomRequest { receiver, success } => {
            let route = state.usrs.get(&receiver.key);
            if success.send(route.cloned()).is_err() {
                eprintln!("RoomRequest response could not be sent user disconnected");
            }
        }
        SocketMessage::GenerateRoom {
            proposer,
            recipient,
        } => {
            let mut room_key: [u8; 64] = [0u8; 64];

            loop{
                thread_rng().fill_bytes(&mut room_key);
                if ! state.rooms.contains_key(&room_key) {
                    break;
                }
            }

            // Generate futures to send room key to users
            let proposer_fut = proposer.send(ServerMessage::RoomAffirmation {
                room_id: Some(room_key),
            });
            let recipient_fut = recipient.send(ServerMessage::RoomAffirmation {
                room_id: Some(room_key),
            });

            let (recipient_result, proposer_result) = join!(recipient_fut, proposer_fut);
        
            if recipient_result.is_ok() || proposer_result.is_ok(){
                state.rooms.insert(room_key, RoomStatus::Idle(Instant::now()));
            }
        }
    }
}
