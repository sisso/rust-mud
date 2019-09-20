use rand::Rng;

use super::comm;
use super::container::Container;
use super::Outputs;
use super::domain::*;
use super::mob::*;
use super::room::RoomId;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct SpawnId(pub u32);

use crate::utils::*;
use crate::game::Ctx;

pub struct SpawnDelay {
    pub min: Second,
    pub max: Second
}

//pub struct SpawnPrefab {
//    pub probability_0_100: u32,
//    pub prefab_id: MobPrefabId
//}

pub struct Spawn {
    pub id: SpawnId,
    pub room_id: RoomId,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefab_id: MobPrefabId,
    pub next: Option<Second>,
    pub mobs_id: Vec<MobId>,
}

pub fn run(ctx: &mut Ctx) {
    for spawn_id in ctx.container.list_spawn() {
        clean_up_dead_mobs(ctx.container, &spawn_id);

        let spawn = ctx.container.get_spawn_by_id_mut(&spawn_id);

        let can_spawn_mobs = spawn.mobs_id.len() < spawn.max as usize;

        match spawn.next {
            Some(next) if next.le(ctx.time.total) && can_spawn_mobs => {
                // spawn mob
                let room_id = spawn.room_id;
                let mob_prefab_id = spawn.prefab_id;
                let mob = ctx.container.instantiate(mob_prefab_id, room_id);
                let mob_id = mob.id;

                debug!("{:?}({:?}) at {:?}", mob.label, mob.id, room_id);

                let spawn_msg = comm::spawn_mob(&mob);

                // update spawn
                let spawn = ctx.container.get_spawn_by_id_mut(&spawn_id);
                spawn.mobs_id.push(mob_id);
                schedule_next_spawn(ctx.time.total, spawn);

                // add outputs
                ctx.outputs.room_all(room_id, spawn_msg);

            },
            Some(_) => {
            },
            None => {
                schedule_next_spawn(ctx.time.total, spawn);
            },
        };
    }
}

fn schedule_next_spawn(now: Second, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let next = Second(rng.gen_range(spawn.delay.min.0, spawn.delay.max.0));
    spawn.next = Some(next + now);

    debug!("scheduling spawn {:?} to {:?}", spawn.id, next);
}

fn clean_up_dead_mobs(container: &mut Container, spawn_id: &SpawnId) {
    let mut remove_list = vec![];
    let spawn = container.get_spawn_by_id(spawn_id);
    for (i, mob_id) in spawn.mobs_id.iter().enumerate() {
        if !container.mobs.exists(&mob_id) {
            remove_list.push(i);
        }
    }

    let spawn = container.get_spawn_by_id_mut(spawn_id);
    for i in remove_list.iter().rev() {
        spawn.mobs_id.remove(*i);
    }
}
