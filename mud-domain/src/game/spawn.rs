use commons::*;
use rand::Rng;

use super::comm;
use super::Outputs;
use super::mob;
use super::mob::*;
use super::room::RoomId;
use logs::*;


use crate::game::Ctx;
use std::collections::HashMap;

type SpawnId = ObjId;

pub struct SpawnDelay {
    pub min: DeltaTime,
    pub max: DeltaTime
}

pub struct Spawn {
    pub id: SpawnId,
    pub room_id: RoomId,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefab_id: MobPrefabId,
    pub next: Option<TotalTime>,
    pub mobs_id: Vec<MobId>,
}

pub struct Spawns {
    spawns: HashMap<SpawnId, Spawn>,
}

impl Spawns {
    pub fn new() -> Self {
        Spawns {
            spawns: HashMap::new(),
        }
    }

    pub fn add(&mut self, spawn: Spawn) -> Option<Spawn> {
        assert!(!self.spawns.contains_key(&spawn.id));
        self.spawns.insert(spawn.id, spawn)
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Spawn> {
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
}

pub fn run(ctx: &mut Ctx) {
    let total_time = ctx.container.time.total;

    for spawn in ctx.container.spawns.list_mut() {
        clean_up_dead_mobs(&mut ctx.container.mobs, spawn);

        match spawn.next {
            Some(next) if next.is_after(total_time) => {
                let can_spawn_mobs = spawn.mobs_id.len() < spawn.max as usize;

                if can_spawn_mobs {
                    // spawn mob
                    let room_id = spawn.room_id;
                    let mob_prefab_id = spawn.prefab_id;
                    let mob = mob::instantiate_from_prefab(
                        &mut ctx.container.objects,
                        &mut ctx.container.mobs,
                        &mut ctx.container.items,
                        mob_prefab_id,
                        room_id
                    );
                    let mob_id = mob.id;

                    debug!("{:?}({:?}) at {:?}", mob.label, mob.id, room_id);

                    let spawn_msg = comm::spawn_mob(&mob);

                    // update spawn
                    spawn.mobs_id.push(mob_id);
                    schedule_next_spawn(total_time, spawn);

                    // add outputs
                    ctx.outputs.room_all(room_id, spawn_msg);
                }
            },
            Some(_) => {},
            None => schedule_next_spawn(total_time, spawn),
        };
    }
}

fn schedule_next_spawn(now: TotalTime, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let next = DeltaTime(rng.gen_range(spawn.delay.min.as_f32(), spawn.delay.max.as_f32()));
    spawn.next = Some(now + next);

    debug!("scheduling spawn {:?} to {:?}", spawn.id, next);
}

// TODO: should be a trigger
fn clean_up_dead_mobs(mobs: &mut MobRepository, spawn: &mut Spawn) {
    spawn.mobs_id.retain(|mob_id| {
       mobs.exists(*mob_id)
    });
}
