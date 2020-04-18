use super::comm;
use super::mob::*;
use super::room::RoomId;
use super::Outputs;
use crate::errors::{Error, Result};
use crate::game::loader::{Loader, StaticId};
use commons::*;
use logs::*;
use rand::Rng;
use std::collections::HashMap;
use crate::game::system::SystemCtx;

type SpawnId = ObjId;

#[derive(Clone, Debug)]
pub struct SpawnDelay {
    pub min: DeltaTime,
    pub max: DeltaTime,
}

impl SpawnDelay {
    pub fn validate(&self) -> Result<()> {
        // TODO: create fixed delay
        if (self.min.as_f32() - self.max.as_f32()).abs() < 0.01 {
            return Err(Error::Error("Min and max time can not be so short".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpawnBuilder {
    pub max: u32,
    pub delay_min: DeltaTime,
    pub delay_max: DeltaTime,
    pub prefab_id: StaticId,
}

impl SpawnBuilder {
    pub fn create_spawn(&self, id: ObjId) -> Spawn {
        Spawn::new(id, self.prefab_id, self.delay_min, self.delay_max)
    }
}

#[derive(Debug, Clone)]
pub struct Spawn {
    pub id: SpawnId,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefab_id: StaticId,
    pub next: TotalTime,
}

impl Spawn {
    pub fn new(id: SpawnId, prefab_id: StaticId, min: DeltaTime, max: DeltaTime) -> Self {
        Spawn {
            id,
            max: 1,
            delay: SpawnDelay { min: min, max: max },
            prefab_id: prefab_id,
            next: TotalTime(0.0),
        }
    }

}

#[derive(Debug)]
pub struct Spawns {
    spawns: HashMap<SpawnId, Spawn>,
    added: Vec<SpawnId>,
}

impl Spawns {
    pub fn new() -> Self {
        Spawns {
            spawns: HashMap::new(),
            added: vec![],
        }
    }

    pub fn add(&mut self, spawn: Spawn) -> Result<()> {
        let _ = spawn.delay.validate()?;
        if self.spawns.contains_key(&spawn.id) {
            Err(Error::ConflictException)
        } else {
            debug!("{:?} spawn added {:?}", spawn.id, spawn);
            let spawn_id = spawn.id;
            self.spawns.insert(spawn_id, spawn);
            self.added.push(spawn_id);
            Ok(())
        }
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Spawn> {
        debug!("{:?} spawn removed", id);
        self.spawns.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Spawn> {
        self.spawns.get(&id)
    }

    pub fn take_added(&mut self) -> Vec<SpawnId> {
        std::mem::replace(&mut self.added, vec![])
    }

    pub fn list_entries_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut Spawn)> + 'a {
        self.spawns.iter_mut()
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Spawn> + 'a {
        self.spawns.values_mut()
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Spawn> {
        self.spawns.get_mut(&id)
    }
}

