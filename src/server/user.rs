use serde::{Serialize, Deserialize};


#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct User {
    pub key: PubKey,
}

type PubKey = [u8; 32];
