use tokio::sync::mpsc::Sender;

use crate::server::{
    messages::{ServerMessage, SocketMessage},
    user::User,
};

pub struct State {
    /// Authenticated user for this instance
    pub user: User,
    /// sender that sends socket messages to the server
    pub com: Sender<SocketMessage>,
    /// sender that sends server messages to this instance
    pub tx: Sender<ServerMessage>,
}
