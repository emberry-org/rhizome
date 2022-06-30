mod certs;
mod control_channel;
mod rendezvous;
mod settings;
mod err;
#[cfg(feature = "certgen")]
pub use certs::regenerate_certs;
pub use settings::Args;

use err::eprinterr_with;
use certs::{load_certs, load_private_key};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{path::PathBuf, sync::Arc};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio_rustls::{rustls, TlsAcceptor};

#[tokio::main(flavor = "current_thread")]
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configure the TlsAcceptor and bind the TcpListener
    let (tcp, tls) = configure_tls(args.tls_port, args.cert, args.key).await?;

    // Configure and bind the udp socket
    let udp = configue_udp(args.udp_port).await?;
    let mut matchmap: HashMap<[u8; 64], SocketAddr> = Default::default();
    let mut udp_buff = [0u8; 64];

    // Prepare a long-running future stream to accept and serve clients.
    loop {
        let acceptor = tls.clone(); // We need a new Acceptor for each client because of TLS connection state
        select! {
            Ok(socket) = tcp.accept() => {
                tokio::spawn(eprinterr_with(control_channel::handle(socket, acceptor), "control_channel")); // Spawn tokio task to handle tls control channel
            }
            Ok((size, addr)) = udp.recv_from(&mut udp_buff) => {
                if size != 64 {
                    eprintln!("udp recv {} bytes from {}", size, addr);
                    continue;
                }
                rendezvous::handle(&udp, &mut matchmap, &udp_buff, addr).await?; // Handle rendezvous service directly since updates to matchmap should be atomic
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
