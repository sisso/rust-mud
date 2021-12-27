use crate::errors::{Error, Result};
use crate::game::domain::Dir;
use crate::game::room::RoomId;
use crate::game::spawn::SpawnBuilder;
use commons::ObjId;
use rand::prelude::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomRoomsSpawnCfg {
    pub amount: u32,
    pub level_min: Option<u32>,
    pub level_max: Option<u32>,
    pub spawn_builder: SpawnBuilder,
}

impl RandomRoomsSpawnCfg {
    pub fn is_valid_for(&self, deep: u32) -> bool {
        self.level_min.map(|min| deep >= min).unwrap_or(true)
            && self.level_max.map(|max| deep <= max).unwrap_or(true)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomRoomsCfg {
    pub entrance_id: RoomId,
    pub entrance_dir: Dir,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub spawns: Vec<RandomRoomsSpawnCfg>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomRoomsState {
    pub id: ObjId,
    pub cfg: RandomRoomsCfg,
    pub generated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomRoomsRepository {
    index: HashMap<ObjId, RandomRoomsState>,
}

impl RandomRoomsRepository {
    pub fn new() -> Self {
        RandomRoomsRepository {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, state: RandomRoomsState) -> Result<()> {
        if self.index.contains_key(&state.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(state.id, state);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<RandomRoomsState> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&RandomRoomsState> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut RandomRoomsState> + 'a {
        self.index.values_mut()
    }
}
