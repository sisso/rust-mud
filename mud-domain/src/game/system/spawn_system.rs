use commons::unwrap_or_continue;
use logs::*;

use crate::errors::*;
use crate::game::comm;
use crate::game::container::Container;
use crate::game::loader::Loader;
use crate::game::mob::MobRepository;
use crate::game::spawn::Spawn;
use crate::game::timer::Timer;
use crate::game::triggers::{Event, EventKind};
use commons::{DeltaTime, TotalTime};
use rand::{thread_rng, Rng};

pub fn run(container: &mut Container) {
    let total_time = container.time.total;

    // schedule all new spawns
    for spawn_id in container.spawns.take_added() {
        let spawn = match container.spawns.get_mut(spawn_id) {
            Some(spawn) => spawn,
            None => {
                warn!("{:?} added spawn not found", spawn_id);
                continue;
            }
        };

        schedule_next_spawn(&mut container.timer, total_time, spawn);
    }

    let mut mob_spawns = vec![];

    // process all already triggered spawns
    for event in container.triggers.list(EventKind::Spawn) {
        let spawn_id = event.get_obj_id();
        let spawn = unwrap_or_continue!(container.spawns.get_mut(spawn_id));

        let can_spawn_mobs = container.ownership.count(spawn.id) < spawn.max as usize;

        if can_spawn_mobs {
            let location_id = if spawn.locations_id.is_empty() {
                container.locations.get(spawn.id)
            } else {
                let index = thread_rng().gen_range(0, spawn.locations_id.len());
                spawn.locations_id.get(index).cloned()
            };

            match location_id {
                Some(location_id) => {
                    let is_valid =
                        container.rooms.exists(location_id) || container.items.exists(location_id);

                    if is_valid {
                        mob_spawns.push((spawn.id, location_id, spawn.prefab_id));
                    } else {
                        warn!(
                            "{:?} Spawn parent {:?} is not a valid room or item.",
                            spawn.id, location_id
                        );
                    }
                }
                None => warn!("{:?} Spawn has no parent", spawn.id),
            };
        } else {
            debug!("{:?} can not spawn, already own max objects", spawn.id);
        }

        schedule_next_spawn(&mut container.timer, total_time, spawn);
    }

    // spawn all mobs
    for (spawn_id, room_id, mob_prefab_id) in mob_spawns {
        let mob_id = match Loader::spawn_at(container, mob_prefab_id, room_id) {
            Ok(mob_id) => mob_id,
            Err(e) => {
                warn!(
                    "{:?} fail to spawn a {:?}: {:?}",
                    spawn_id, mob_prefab_id, e
                );
                continue;
            }
        };

        let mob_label = container.labels.get_label_f(mob_id);

        debug!("{:?} spawn created {:?} at {:?}", spawn_id, mob_id, room_id);

        // TODO: move to ownership system
        let spawn_msg = comm::spawn_mob(mob_label);

        // update spawn
        container.ownership.set_owner(mob_id, spawn_id);

        // add outputs
        container.outputs.broadcast(None, room_id, spawn_msg);
    }
}

fn schedule_next_spawn(timer: &mut Timer, now: TotalTime, spawn: &mut Spawn) {
    let mut rng = rand::thread_rng();
    let range = rng.gen_range(
        spawn.delay.min.as_seconds_f32(),
        spawn.delay.max.as_seconds_f32(),
    );
    let next = now + DeltaTime(range);
    spawn.next = next;
    timer.schedule(
        next,
        Event::Obj {
            kind: EventKind::Spawn,
            obj_id: spawn.id,
        },
    );
    debug!("{:?} scheduling spawn at {:?}", spawn.id, spawn.next);
}
