use serde::{Serialize, Deserialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::server::TlsStream;

use crate::server::user::User;

use std::io::{self, ErrorKind};

#[derive(Clone, Serialize, Deserialize)]
pub enum Response {
    HasRoute(User),
    NoRoute(User),
    WantsRoom(User, Option<String>),
    AcceptedRoom(Option<[u8; 32]>)
}

impl Response {
    pub async fn send_with<T>(self, tls: &mut BufReader<TlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut buf = vec![];
        todo!("packet serialization");

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&buf).await
    }
}
