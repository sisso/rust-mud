use logs::*;
use commons::unwrap_or_continue;

use crate::game::system::SystemCtx;
use crate::game::loader::Loader;
use crate::game::comm;
use commons::{TotalTime, DeltaTime};
use crate::game::spawn::Spawn;
use crate::game::mob::MobRepository;
use rand::Rng;
use crate::errors::*;
use crate::game::timer::Timer;
use crate::game::triggers::{Event, EventKind};

pub fn run(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;

    // schedule all new spawns
    for spawn_id in ctx.container.spawns.take_added() {
        let spawn = match ctx.container.spawns.get_mut(spawn_id) {
            Some(spawn) => spawn,
            None => {
                warn!("{:?} added spawn not found", spawn_id);
                continue;
            },
        };

        schedule_next_spawn(&mut ctx.container.timer, total_time, spawn);
    }

    let mut mob_spawns = vec![];

    // process all already triggered spawns
    for event in ctx.container.triggers.list(EventKind::Spawn) {
        let spawn_id = event.get_obj_id();
        let mut spawn = unwrap_or_continue!(ctx.container.spawns.get_mut(spawn_id));

        let can_spawn_mobs = ctx.container.ownership.count(spawn.id) < spawn.max as usize;

        println!("can_spawn_mobs {}", can_spawn_mobs);

        if can_spawn_mobs {
            match ctx.container.locations.get(spawn.id) {
                Some(location_id) => {
                    let is_valid = ctx.container.rooms.exists(location_id)
                        || ctx.container.items.exists(location_id);

                    if is_valid {
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
        } else{
            debug!("{:?} can not spawn, already own max objects", spawn.id); 
        }

        schedule_next_spawn(&mut ctx.container.timer, total_time, spawn);
    }

    // spawn all mobs
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
        ctx.container.ownership.set_owner(mob_id, spawn_id);

        // add outputs
        ctx.outputs.broadcast(None, room_id, spawn_msg);
    }
}

// TODO: move to spawn
fn schedule_next_spawn(timer: &mut Timer, now: TotalTime, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let range = rng.gen_range(spawn.delay.min.as_f32(), spawn.delay.max.as_f32());
    let next = now + DeltaTime(range);
    spawn.next = Some(next);
    timer.schedule(next, Event::Obj { kind: EventKind::Spawn, obj_id: spawn.id });
    debug!("{:?} scheduling spawn at {:?}", spawn.id, spawn.next);
}

// // TODO: should be a trigger
// fn clean_up_dead_mobs(spawn_id: ObjId, mobs: &mut MobRepository, ownership: &mut Ownership) {
//     let mut clean_list = vec![];
//     for obj_id in ownership.list(spawn_id) {
//         if !mobs.exists(obj_id) {
//             
//         }
//         spawn.mobs_id.retain(|mob_id| mobs.exists(*mob_id));
//     }
// }
