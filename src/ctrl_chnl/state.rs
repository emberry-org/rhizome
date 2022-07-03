use tokio::sync::mpsc::Sender;

use crate::server::{
    messages::{ServerMessage, SocketMessage},
    user::User,
};

pub struct State {
    /// Authenticated user for this instance
    pub user: User,
    /// sender that sends server messages to this instance
    pub tx: Sender<ServerMessage>,
}
