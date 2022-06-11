mod certs;
mod settings;
pub use settings::Args;
#[cfg(feature = "certgen")]
pub use certs::regenerate_certs;

use certs::{load_certs, load_private_key};
use std::io::{self, ErrorKind, Write};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

#[tokio::main(flavor = "current_thread")]
pub async fn run(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", args.tls_port);
    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let certs = load_certs(args.cert)?;
        // Load private key.
        let key = load_private_key(args.key)?;

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
    // Prepare a long-running future stream to accept and serve clients.
    loop {
        println!("awaiting new client");
        let acceptor = tls_acceptor.clone(); // We need a new Acceptor for each client because of TLS connection state
        let (socket, _) = tcp.accept().await?; // Accept TCP client

        // Create future for handleing TLS handshake
        let fut = async move {
            let mut tls = acceptor.accept(socket).await?; // Perform TLS Handshake
            println!("tls established");

            // ---------------- Demo specific payload starts here ----------------
            // read stream until 0x00 occurs then repeat the exact same sequence to the client (echo)
            let mut buf_tls = BufReader::new(&mut tls);
            let mut buf = vec![];

            match buf_tls.read_until(0, &mut buf).await {
                Ok(_) => (),
                Err(err) => {
                    if err.kind() == ErrorKind::ConnectionReset {
                        println!("connection closed");
                        return io::Result::Ok(()); // Return OK on Connection reset
                    } else {
                        return Err(err);
                    }
                }
            }
            tls.write_all(&buf).await?;
            std::io::stdout().write_all(&buf)?; // Write echoed message to stdout
                                                // ---------------- Demo specific payload ends here ----------------

            io::Result::Ok(()) // Connection will be automatically closed when future exits
        };

        // Execute the future
        tokio::spawn(async move {
            if let Err(err) = fut.await {
                eprintln!("{:?}", err); // Print error and keep going (Likely tls handshake failed/tcp connection failed)
                                        // One may want to differentiate this error in production and take different actions
            }
        });
    }
}
