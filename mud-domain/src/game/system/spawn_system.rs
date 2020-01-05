use logs::*;

use crate::game::system::SystemCtx;
use crate::game::loader::Loader;
use crate::game::comm;
use commons::{TotalTime, DeltaTime};
use crate::game::spawn::Spawn;
use crate::game::mob::MobRepository;
use rand::Rng;
use crate::errors::*;

pub fn run(ctx: &mut SystemCtx) {
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

                let candidate_location = ctx.container.locations.get(spawn.id);
                match candidate_location {
                    Some(location_id) => {
                        let valid = ctx.container.rooms.exists(location_id)
                            || ctx.container.items.exists(location_id);

                        if valid {
                            mob_spawns.push((spawn.id, location_id, spawn.prefab_id));
                        } else {
                            warn!(
                                "Spawn {:?} parent {:?} is not a valid room or item.",
                                spawn.id, location_id
                            );
                        }
                    }
                    None => warn!("Spawn {:?} has no parent", spawn.id),
                };
            }

            Some(_next) => {}

            None => schedule_next_spawn(total_time, spawn),
        };
    }

    for (spawn_id, room_id, mob_prefab_id) in mob_spawns {
        let mob_id = match Loader::spawn_at(&mut ctx.container, mob_prefab_id, room_id) {
            Ok(mob_id) => mob_id,
            Err(e) => {
                warn!(
                    "{:?} fail to spawn a {:?}: {:?}",
                    spawn_id, mob_prefab_id, e
                );
                continue;
            }
        };

        let mob_label = ctx.container.labels.get_label_f(mob_id);

        debug!("{:?} spawn created {:?} at {:?}", spawn_id, mob_id, room_id);

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
