use tokio::sync::{mpsc::Sender, oneshot};

use smoke::User;

pub struct RoomProposal {
    pub proposer: User,
    pub proposer_tx: Sender<ServerMessage>,
}

pub enum ServerMessage {
    RoomProposal { proposal: RoomProposal },
    RoomAffirmation { room_id: Option<[u8; 32]> },
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
    GenerateRoom {
        proposer: Sender<ServerMessage>,
        recipient: Sender<ServerMessage>,
    },
}
