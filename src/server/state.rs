use std::collections::HashMap;

use tokio::sync::mpsc::Sender;

use crate::rendezvous::RoomStatus;

use super::messages::ServerMessage;

pub struct State {
    pub usrs: HashMap<[u8; 32], Sender<ServerMessage>>,
    pub rooms: HashMap<[u8; 32], RoomStatus>,
}
