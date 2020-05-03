use super::domain::Dir;
use crate::errors::{Error, Result};
use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type RoomId = ObjId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Room {
    pub id: RoomId,
    pub exits: Vec<(Dir, RoomId)>,
    /// Used to implement Airlock or cave exit
    pub can_exit: bool,
}

impl Room {
    pub fn new(id: RoomId) -> Self {
        Room {
            id,
            exits: vec![],
            can_exit: false,
        }
    }
}

impl Room {
    pub fn get_exit(&self, dir: &Dir) -> Option<RoomId> {
        self.exits.iter().find(|e| e.0 == *dir).map(|i| i.1)
    }

    pub fn get_exit_for(&self, room_id: RoomId) -> Option<Dir> {
        self.exits
            .iter()
            .find(|(_, id)| *id == room_id)
            .map(|(dir, _)| dir)
            .cloned()
    }

    pub fn remove_exit(&mut self, room_id: RoomId, dir: Dir) -> Result<()> {
        self.exits
            .iter()
            .position(|(i_dir, i_id)| *i_dir == dir && *i_id == room_id)
            .map(|position| {
                self.exits.remove(position);
            })
            .ok_or(Error::NotFoundFailure)
    }
}

#[derive(Debug, Clone)]
pub struct RoomRepository {
    index: HashMap<RoomId, Room>,
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
        debug!(
            "adding portal {:?} to {:?} from {:?}",
            room1_id, room2_id, dir
        );

        let room2 = self.index.get_mut(&room2_id).unwrap();
        room2.exits.push((dir.inv(), room1_id));
        debug!(
            "adding portal {:?} to {:?} from {:?}",
            room2_id,
            room1_id,
            dir.inv()
        );
    }

    pub fn get_portals(&self, room_id: RoomId) -> Result<&Vec<(Dir, RoomId)>> {
        Ok(&self.get(room_id).unwrap().exits)
    }

    pub fn exists(&self, id: RoomId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn update<F>(&mut self, room_id: RoomId, f: F) -> Result<()>
    where
        F: FnOnce(&mut Room),
    {
        self.index
            .get_mut(&room_id)
            .ok_or(Error::NotFoundFailure)
            .map(|room| {
                f(room);
                debug!("{:?} updated", room);
            })
    }

    pub fn remove_portal(&mut self, room1_id: RoomId, room2_id: RoomId, dir: Dir) -> Result<()> {
        self.index
            .get_mut(&room1_id)
            .ok_or(Error::NotFoundFailure)
            .and_then(|room| room.remove_exit(room2_id, dir))?;

        debug!(
            "remove portal {:?} to {:?} from {:?}",
            room1_id, room2_id, dir
        );

        self.index
            .get_mut(&room2_id)
            .ok_or(Error::NotFoundFailure)
            .and_then(|room| room.remove_exit(room1_id, dir.inv()))?;

        debug!(
            "remove portal {:?} to {:?} from {:?}",
            room2_id,
            room1_id,
            dir.inv()
        );
        Ok(())
    }

    pub fn exists_exits(&self, room1_id: RoomId, room2_id: RoomId) -> Option<Dir> {
        self.index
            .get(&room1_id)
            .and_then(|room| room.get_exit_for(room2_id))
    }
}

impl SnapshotSupport for RoomRepository {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            if id.is_static() {
                continue;
            }
            snapshot.add(id.as_u32(), "room", json!(comp));
        }
    }
}
