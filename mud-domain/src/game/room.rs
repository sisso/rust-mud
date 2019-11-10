use std::collections::HashMap;
use logs::*;
use super::domain::Dir;
use commons::ObjId;

pub type RoomId = ObjId;

#[derive(Clone, Debug)]
pub struct Room {
    pub id: RoomId,
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
        debug!("{:?} add room {:?}", room.id, room);
        self.index.insert(room.id, room);
    }

    pub fn get(&self, id: RoomId) -> Result<&Room, ()> {
        self.index.get(&id).ok_or(())
    }

    pub fn get_mut(&mut self, id: RoomId) -> Result<&mut Room, ()> {
        self.index.get_mut(&id).ok_or(())
    }

    pub fn is_room(&self, id: RoomId) -> bool {
        self.index.contains_key(&id)
    }
}
