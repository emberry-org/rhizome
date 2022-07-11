use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_rustls::server::TlsStream;

use crate::server::user::User;

use std::io;

#[derive(Clone, Serialize, Deserialize)]
pub enum RhizMessage {
    HasRoute(User),
    NoRoute(User),
    WantsRoom(User),
    AcceptedRoom(Option<[u8; 32]>),
    ServerError(String),
}

impl RhizMessage {
    pub async fn send_with<T>(self, tls: &mut BufReader<TlsStream<T>>) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // use 192 byte buffer since we restrict wants room string to max 128 characters
        let bytes = postcard::to_vec_cobs::<Self, 64>(&self)
            .expect("there was a big oopsie during developement!
            serialization size for RhizMessage was bigger then the specified buffer size");

        #[cfg(feature = "debug")]
        println!("sent msg");
        tls.write_all(&bytes).await
    }
}
