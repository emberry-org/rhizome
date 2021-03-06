use tokio::sync::mpsc::Sender;

use crate::server::user::User;

pub enum ServerMessage {
    RoomProposal { sender: User, msg: Option<String> },
}

pub enum SocketMessage {
    SubscribeUser {
        user: User,
        tx: Sender<ServerMessage>,
    },
    Disconnect {
        user: User,
    },
}
