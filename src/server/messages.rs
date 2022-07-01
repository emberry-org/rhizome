use tokio::sync::{mpsc::Sender, oneshot};

use crate::server::user::User;

pub enum ServerMessage {
    RoomProposal { sender: User, msg: Option<String> },
}

pub enum SocketMessage {
    RoomRequest {
        receiver: User,
        success: oneshot::Sender<Option<Sender<ServerMessage>>>,
    },
    SubscribeUser {
        user: User,
        tx: Sender<ServerMessage>,
    },
    Disconnect {
        user: User,
    },
}
