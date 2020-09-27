use super::comm;
use super::mob::*;
use super::room::RoomId;
use crate::errors::{Error, Result};
use crate::game::loader::{dto::StaticId, Loader};
use crate::game::location::LocationId;
use commons::*;
use logs::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type SpawnId = ObjId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpawnDelay {
    pub min: DeltaTime,
    pub max: DeltaTime,
}

impl SpawnDelay {
    pub fn validate(&self) -> Result<()> {
        // TODO: create fixed delay
        if (self.min.as_seconds_f32() - self.max.as_seconds_f32()).abs() < 0.01 {
            return Err(Error::Error(
                "Min and max time can not be too short".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpawnBuilder {
    pub max: u32,
    pub delay_min: DeltaTime,
    pub delay_max: DeltaTime,
    pub prefab_id: StaticId,
    pub next: Option<TotalTime>,
}

impl SpawnBuilder {
    pub fn create_spawn(&self, id: ObjId) -> Spawn {
        Spawn::new(id, self.prefab_id, self.max, self.delay_min, self.delay_max)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Spawn {
    pub id: SpawnId,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefab_id: StaticId,
    pub next: TotalTime,
    /// zones and rooms are valid, when empty, parent objects is used
    pub locations_id: Vec<LocationId>,
}

impl Spawn {
    pub fn new(
        id: SpawnId,
        prefab_id: StaticId,
        max: u32,
        min_time: DeltaTime,
        max_time: DeltaTime,
    ) -> Self {
        Spawn {
            id,
            max: max,
            delay: SpawnDelay {
                min: min_time,
                max: max_time,
            },
            prefab_id: prefab_id,
            next: TotalTime(0.0),
            locations_id: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spawns {
    index: HashMap<SpawnId, Spawn>,
    // TODO: remove added
    added: Vec<SpawnId>,
}

impl Spawns {
    pub fn new() -> Self {
        Spawns {
            index: HashMap::new(),
            added: vec![],
        }
    }

    pub fn add(&mut self, spawn: Spawn) -> Result<()> {
        let _ = spawn.delay.validate()?;
        if self.index.contains_key(&spawn.id) {
            Err(Error::ConflictException)
        } else {
            debug!("{:?} spawn added {:?}", spawn.id, spawn);
            let spawn_id = spawn.id;
            self.index.insert(spawn_id, spawn);
            self.added.push(spawn_id);
            Ok(())
        }
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Spawn> {
        debug!("{:?} spawn removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Spawn> {
        self.index.get(&id)
    }

    pub fn take_added(&mut self) -> Vec<SpawnId> {
        std::mem::replace(&mut self.added, vec![])
    }

    pub fn list_entries_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut Spawn)> + 'a {
        self.index.iter_mut()
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Spawn> + 'a {
        self.index.values_mut()
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Spawn> {
        self.index.get_mut(&id)
    }
}
