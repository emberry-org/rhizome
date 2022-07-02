use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::server::TlsStream;

use crate::server::user::User;

use std::io::{self, ErrorKind};

#[derive(Clone)]
pub enum Response {
    HasRoute(User),
    NoRoute(User),
}

impl Response {
    pub async fn send_with<T>(self, tls: &mut BufReader<TlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut buf = vec![];
        match self {
            Self::HasRoute(usr) | Self::NoRoute(usr) => todo!("packet serialization"),
        };

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&buf).await
    }
}
