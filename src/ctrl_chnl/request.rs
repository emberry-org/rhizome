use crate::user::User;

pub enum Request{
    RoomRequest(User),
    Heartbeat,
    Shutdown,
}