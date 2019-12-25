use super::comm;
use super::mob::*;
use super::room::RoomId;
use super::Outputs;
use commons::*;
use logs::*;
use rand::Rng;
use std::collections::HashMap;
use crate::game::loader::{StaticId, Loader};
use crate::errors::{Result, Error};
use crate::game::container::Ctx;

type SpawnId = ObjId;

#[derive(Debug)]
pub struct SpawnDelay {
    pub min: DeltaTime,
    pub max: DeltaTime,
}

impl SpawnDelay {
    pub fn validate(&self) -> Result<()> {
        // TODO: create fixed delay
        if (self.min.as_f32() - self.max.as_f32()).abs() < 0.01 {
            return Err(Error::IllegalArgumentMsg { msg: "Min and max time can not be so short".to_string() });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Spawn {
    pub id: SpawnId,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefab_id: StaticId ,
    next: Option<TotalTime>,
    mobs_id: Vec<MobId>,
}

impl Spawn {
    pub fn new(id: SpawnId, prefab_id: StaticId, min: DeltaTime, max: DeltaTime) -> Self {
        Spawn {
            id,
            max: 1,
            delay: SpawnDelay { min: min, max: max },
            prefab_id: prefab_id,
            next: None,
            mobs_id: vec![]
        }
    }
}

#[derive(Debug)]
pub struct Spawns {
    spawns: HashMap<SpawnId, Spawn>,
}

impl Spawns {
    pub fn new() -> Self {
        Spawns {
            spawns: HashMap::new(),
        }
    }

    pub fn add(&mut self, spawn: Spawn) -> Result<()> {
        assert!(!self.spawns.contains_key(&spawn.id));
        let _ = spawn.delay.validate()?;
        if self.spawns.contains_key(&spawn.id) {
            Err(Error::Conflict)
        } else {
            debug!("{:?} spawn added {:?}", spawn.id, spawn);
            self.spawns.insert(spawn.id, spawn);
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

    //    pub fn list(&self) -> Vec<&Spawn> {
    //        unimplemented!()
    //    }

    pub fn list_entries_mut<'a>(&'a mut self) -> impl Iterator<Item = (&ObjId, &mut Spawn)> + 'a {
        self.spawns.iter_mut()
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = &mut Spawn> + 'a {
        self.spawns.values_mut()
    }

    fn get_mut(&mut self, id: ObjId) -> Option<&mut Spawn> {
        self.spawns.get_mut(&id)
    }

    pub fn add_mob_id(&mut self, spawn_id: SpawnId, mob_id: MobId) {
        self.spawns.get_mut(&spawn_id).unwrap().mobs_id.push(mob_id);
    }
}

// TODO: move to system/spawn (create systems first)
pub fn run(ctx: &mut Ctx) {
    let total_time = ctx.container.time.total;

    let mut mob_spawns = vec![];

    for spawn in ctx.container.spawns.list_mut() {
        clean_up_dead_mobs(&mut ctx.container.mobs, spawn);
        let can_spawn_mobs = || spawn.mobs_id.len() < spawn.max as usize;

        match spawn.next {
            Some(next) if next.is_before(total_time) && !can_spawn_mobs() => {
                // when full, just schedule next spawn
                schedule_next_spawn(total_time, spawn);
            }

            Some(next) if next.is_before(total_time) => {
                schedule_next_spawn(total_time, spawn);

                match ctx.container.locations.get(spawn.id) {
                    Some(location_id) if ctx.container.rooms.exists(location_id) => {
                        mob_spawns.push((spawn.id, location_id, spawn.prefab_id))
                    }

                    other => {
                        warn!("Spawn {:?} parent {:?} is not a valid room.", spawn.id, other);
                    }
                }
            }

            Some(_next) => {},

            None => schedule_next_spawn(total_time, spawn),
        };
    }

    for (spawn_id, room_id, mob_prefab_id) in mob_spawns {
        let mob_id = match Loader::spawn_at(&mut ctx.container, mob_prefab_id, room_id) {
            Ok(mob_id) => mob_id,
            Err(e) => {
                warn!("{:?} fail to spawn a {:?}: {:?}", spawn_id, mob_prefab_id, e);
                continue;
            }
        };

       let mob_label = ctx.container.labels.get_label_f(mob_id);

        debug!("{:?} spawn created mob {:?} at {:?}", spawn_id, mob_id, room_id);

        // TODO: move to ownership system
        let spawn_msg = comm::spawn_mob(mob_label);

        // update spawn
        ctx.container.spawns.add_mob_id(spawn_id, mob_id);

        // add outputs
        ctx.outputs.broadcast(None, room_id, spawn_msg);
    }
}

// TODO: move to spawn
fn schedule_next_spawn(now: TotalTime, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let range = rng.gen_range(spawn.delay.min.as_f32(), spawn.delay.max.as_f32());
    let next = DeltaTime(range);
    spawn.next = Some(now + next);

    debug!("{:?} scheduling spawn at {:?}", spawn.id, spawn.next);
}

// TODO: should be a trigger
fn clean_up_dead_mobs(mobs: &mut MobRepository, spawn: &mut Spawn) {
    spawn.mobs_id.retain(|mob_id| mobs.exists(*mob_id));
}
