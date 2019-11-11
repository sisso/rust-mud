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

    pub fn add_portal(&mut self, room1_id: RoomId, room2_id: RoomId, dir: Dir) {
        let room1 = self.index.get_mut(&room1_id).unwrap();
        room1.exits.push((dir, room2_id));

        let room2 = self.index.get_mut(&room2_id).unwrap();
        room2.exits.push((dir.inv(), room1_id));
    }

    pub fn is_room(&self, id: RoomId) -> bool {
        self.index.contains_key(&id)
    }
}
