use crate::server::user::User;

pub enum Request{
    RoomRequest(User),
    Heartbeat,
    Shutdown,
}