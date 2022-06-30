pub enum User {
    Guest,
    Authenticated(PubKey), //
}

type PubKey = [u8; 32];
