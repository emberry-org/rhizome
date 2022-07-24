use tokio::sync::{mpsc::Sender, oneshot};

use smoke::{User, messages::RoomId};

pub struct RoomProposal {
    pub proposer: User,
    pub proposer_tx: Sender<ServerMessage>,
}

pub enum ServerMessage {
    RoomProposal { proposal: RoomProposal },
    RoomAffirmation { room_id: Option<RoomId> , usr: User},
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
        proposer_tx: Sender<ServerMessage>,
        proposer: User,
        recipient_tx: Sender<ServerMessage>,
        recipient: User,
    },
}
