use crate::game::loader::StaticId;
use crate::game::room::RoomId;
use commons::ObjId;

pub struct Config {
    pub initial_room: Option<RoomId>,
    pub avatar_id: Option<StaticId>,
    pub money_id: Option<StaticId>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_room: None,
            avatar_id: None,
            money_id: None,
        }
    }
}
