use std::io::{self, ErrorKind, Write};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

pub async fn handle(socket: (TcpStream, SocketAddr), acceptor: TlsAcceptor) -> io::Result<()> {
    let stream = socket.0;

    // Create future for handleing TLS handshake
    let mut tls = acceptor.accept(stream).await?; // Perform TLS Handshake
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
                return Ok(()); // Return OK on Connection reset
            } else {
                return Err(err);
            }
        }
    }

    tls.write_all(&buf).await?;
    std::io::stdout().write_all(&buf)?; // Write echoed message to stdout

    // ---------------- Demo specific payload ends here ----------------

    Ok(())
}
