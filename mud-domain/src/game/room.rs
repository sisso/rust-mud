use std::collections::HashMap;
use logs::*;
use super::domain::Dir;
use commons::{ObjId, UResult, UERR, UOK};

pub type RoomId = ObjId;

#[derive(Clone, Debug)]
pub struct Room {
    pub id: RoomId,
    pub exits: Vec<(Dir, RoomId)>,
    pub is_airlock: bool,
}

impl Room {
    pub fn new(id: RoomId) -> Self {
        Room {
            id,
            exits: vec![],
            is_airlock: false
        }
    }
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
        debug!("{:?} added", room);
        self.index.insert(room.id, room);
    }

    pub fn get(&self, id: RoomId) -> Option<&Room> {
        self.index.get(&id)
    }

    pub fn add_portal(&mut self, room1_id: RoomId, room2_id: RoomId, dir: Dir) {
        let room1 = self.index.get_mut(&room1_id).unwrap();
        room1.exits.push((dir, room2_id));

        let room2 = self.index.get_mut(&room2_id).unwrap();
        room2.exits.push((dir.inv(), room1_id));
    }

    pub fn exists(&self, id: RoomId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn update<F>(&mut self, room_id: RoomId, f: F) -> UResult
        where F: FnOnce(&mut Room) {

        if let Some(room) = self.index.get_mut(&room_id) {
            f(room);
            debug!("{:?} updated", room);
            UOK
        } else {
            UERR
        }
    }
}
