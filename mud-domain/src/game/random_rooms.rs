use crate::errors::{Error, Result};
use std::collections::HashMap;
use crate::game::room::RoomId;
use crate::game::domain::Dir;
use commons::ObjId;

#[derive(Clone, Debug)]
pub struct RandomRoomsCfg {
    pub id: ObjId,
    pub entrance_id: RoomId,
    pub entrance_dir: Dir,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct RandomRoomsState {
    pub cfg: RandomRoomsCfg,
    pub generated: bool,
}

impl RandomRoomsState {
    pub fn new(cfg: RandomRoomsCfg) -> Self {
        RandomRoomsState {
            cfg,
            generated: false,
        }
    }
}


#[derive(Clone, Debug)]
pub struct RandomRoomsRepository {
    index: HashMap<ObjId, RandomRoomsState>,
}

impl RandomRoomsRepository {
    pub fn new() -> Self {
        RandomRoomsRepository {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, cfg: RandomRoomsCfg) -> Result<()> {
        if self.index.contains_key(&cfg.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(cfg.id, RandomRoomsState::new(cfg));
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<RandomRoomsCfg> {
        self.index.remove(&id).map(|state| state.cfg)
    }

    pub fn get(&self, id: ObjId) -> Option<&RandomRoomsCfg> {
        self.index.get(&id).map(|state| &state.cfg)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_states_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut RandomRoomsState> + 'a {
        self.index.values_mut()
    }
}

