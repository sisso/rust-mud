use crate::game::room::RoomId;
use commons::ObjId;

pub struct Config {
    pub initial_room: RoomId,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_room: ObjId(0),
        }
    }
}
