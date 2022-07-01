
#[derive(Clone, Copy)]
pub struct User {
    pub key: PubKey,
}

type PubKey = [u8; 32];
