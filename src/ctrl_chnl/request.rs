use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};
use tokio_rustls::server::TlsStream;

use crate::server::user::User;


pub const EMB_MESSAGE_BUF_SIZE: usize = 64;
pub type EmbMessageBuf = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub enum EmbMessage {
    Room(User),
    Accept(bool),
    Heartbeat,
    Shutdown,
}

impl EmbMessage {
    pub async fn recv_req<T>(
        tls: &mut BufReader<TlsStream<T>>,
        buf: &mut EmbMessageBuf,
    ) -> io::Result<EmbMessage>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // 0 means EOF so we shutdown the connection
        if 0 == tls.read_until(0, buf).await? {
            return Ok(EmbMessage::Shutdown);
        }

        postcard::from_bytes_cobs(buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
