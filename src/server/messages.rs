use tokio::sync::{mpsc::Sender, oneshot};

use crate::server::user::User;

pub struct RoomProposal {
    pub proposer: User,
    pub proposal: Option<String>,
    pub proposer_tx: Sender<ServerMessage>,
}

pub enum ServerMessage {
    RoomProposal { proposal: RoomProposal },
    ProposalResponse { room_id: Option<[u8; 64]> },
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
