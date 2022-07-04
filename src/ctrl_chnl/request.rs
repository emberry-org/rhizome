use serde::{Deserialize, Serialize};

use crate::server::user::User;

#[derive(Serialize, Deserialize)]
pub enum Request{
    Room(User),
    Accept(bool),
    Heartbeat,
    Shutdown,
}