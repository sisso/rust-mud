use super::domain::*;
use crate::game::controller::Output;

use rand::Rng;
use super::comm;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct SpawnId(pub u32);

pub struct SpawnDelay {
    pub min: Seconds,
    pub max: Seconds
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
    pub next: Option<Seconds>,
    pub mobs_id: Vec<MobId>,
}

pub fn run(container: &mut Container, outputs: &mut Vec<Output>) {
    let time = container.get_time().clone();

    for spawn_id in container.list_spawn() {
        let spawn = container.get_spawn_by_id(&spawn_id);

        let can_spawn_mobs = spawn.mobs_id.len() < spawn.max as usize;

        match spawn.next {
            Some(next) if next.0 <= time.0 && can_spawn_mobs => {
                // spawn mob
                let room_id = spawn.room_id.0;
                let mob_prefab_id = spawn.prefab_id.clone();
                let mob_id = container.next_mob_id();

                let prefab = container.get_mob_prefab(&mob_prefab_id);
                let mob = Mob {
                    id: mob_id,
                    room_id: room_id,
                    label: prefab.label.clone(),
                    is_avatar: false
                };

                println!("spawn - spawning {:?}({}) at {}", mob.label, mob.id, room_id);

                let spawn_msg = comm::spawn_mob(&mob);

                container.add_mob(mob);

                // update spawn
                let spawn = container.get_spawn_by_id(&spawn_id);
                spawn.mobs_id.push(MobId(mob_id));
                schedule_next_spawn(&time, spawn);

                // add outputs
                outputs.push(Output::room_all(room_id, spawn_msg));

            },
            Some(_) => {
            },
            None => {
                schedule_next_spawn(&time, spawn);
            },
        };
    }
}

fn schedule_next_spawn(now: &Seconds, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let next = rng.gen_range(spawn.delay.min.0, spawn.delay.max.0);
    spawn.next = Some(Seconds(next + now.0));

    println!("spawn - scheduling spawn {:?} to {}", spawn.id, next);
}
