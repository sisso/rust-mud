use std::collections::HashMap;

use super::domain::Dir;
use crate::game::obj::ObjId;

pub type RoomId = ObjId;

#[derive(Clone, Debug)]
pub struct Room {
    pub id: RoomId,
    pub label: String,
    pub desc: String,
    pub exits: Vec<(Dir, RoomId)>,
}

impl Room {
    pub fn get_exit(&self, dir: &Dir) -> Option<RoomId> {
        self.exits
            .iter()
            .find(|e| e.0 == *dir)
            .map(|i| i.1)
    }
}

pub struct RoomRepository {
    index: HashMap<RoomId, Room>
}

impl RoomRepository {
    pub fn new() -> Self {
        RoomRepository {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, room: Room) {
        if self.index.contains_key(&room.id) {
            panic!("room already exists");
        }
        self.index.insert(room.id, room);
    }

    pub fn get(&self, id: &RoomId) -> &Room {
        self.index.get(id).unwrap()
    }

    pub fn is_room(&self, id: RoomId) -> bool {
        self.index.contains_key(&id)
    }
}
