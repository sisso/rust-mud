use crate::errors::Result;
use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::labels::{Label, Labels};
use crate::game::location::Locations;
use crate::game::obj::Objects;
use crate::game::random_rooms::{RandomRoomsCfg, RandomRoomsSpawnCfg};
use crate::game::random_rooms_generator::{LevelGrid, RandomLevels, RandomLevelsCfg};
use crate::game::room::{Room, RoomId, RoomRepository};
use crate::game::spawn::Spawns;
use crate::game::system::SystemCtx;
use commons::ObjId;
use logs::*;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::io::repeat;

pub fn run(_ctx: &mut SystemCtx) {}

/// it is implemented in a way that we can re-generate
pub fn init(container: &mut Container) {
    let random_rooms_repo = &mut container.random_rooms;
    let objects = &mut container.objects;
    let rooms = &mut container.rooms;
    let labels = &mut container.labels;
    let locations = &mut container.locations;
    let spawns = &mut container.spawns;

    for rr in random_rooms_repo.list_states_mut() {
        if rr.generated {
            continue;
        }

        info!("{:?} generating random rooms", rr.cfg.id);

        let mut cfg = RandomLevelsCfg {
            rng: &mut rr.rng,
            width: rr.cfg.width as usize,
            height: rr.cfg.height as usize,
            portal_prob: 0.5,
            deep_levels: rr.cfg.levels,
        };

        let levels = RandomLevels::new(&mut cfg);

        let mut previous_down: Option<usize> = None;
        let mut previous_rooms_ids: Option<Vec<ObjId>> = None;

        for (deep, rooms_grid) in levels.levels.iter().enumerate() {
            let rooms_ids = match create_rooms(objects, rooms, labels, rooms_grid) {
                Err(_err) => {
                    warn!(
                        "{:?} error when generating rooms from grid {:?}",
                        rr.cfg.id, err
                    );
                    continue;
                }
                Ok(ids) => ids,
            };

            if deep == 0 {
                connect_rooms_to_entrance(
                    rooms,
                    rr.cfg.entrance_id,
                    rr.cfg.entrance_dir,
                    &rooms_grid,
                    &rooms_ids,
                )
                .unwrap();

                // set down portal
                assert!(rooms_grid.up_portal.is_none());
            } else {
                // resolve up and down portals
                let up_index = rooms_grid.up_portal.unwrap();
                let down_index = previous_down.unwrap();

                let up_id = rooms_ids[up_index];
                let down_id = previous_rooms_ids.unwrap()[down_index];

                rooms.add_portal(down_id, up_id, Dir::D);

                // update next down portal
                previous_down = rooms_grid.down_portal;
            }

            let valid_spawns = rr
                .cfg
                .spawns
                .iter()
                .filter(|spawn| spawn.is_valid_for(deep as u32))
                .collect();

            create_spawns(
                rr.cfg.id,
                &mut rr.rng,
                objects,
                locations,
                spawns,
                &rooms_ids,
                &valid_spawns,
            )
            .unwrap();

            // set variables for next iteration
            previous_down = rooms_grid.down_portal;
            previous_rooms_ids = Some(rooms_ids);
        }

        rr.generated = true;
    }
}

fn create_spawns(
    _rr_id: ObjId,
    rng: &mut StdRng,
    objects: &mut Objects,
    locations: &mut Locations,
    spawns: &mut Spawns,
    rooms_id: &Vec<RoomId>,
    spawns_cfg: &Vec<&RandomRoomsSpawnCfg>,
) -> Result<()> {
    let mut availables = rooms_id.clone();

    for spawn in spawns_cfg {
        for _i in 0..spawn.amount {
            if availables.is_empty() {
                break;
            }

            // find available room
            let candidate_index = rng.gen_range(0, availables.len());
            let room_id = availables.remove(candidate_index);

            let spawn_id = objects.create();
            spawns
                .add(spawn.spawn_builder.create_spawn(spawn_id))
                .unwrap();

            locations.set(spawn_id, room_id);

            trace!(
                "{:?} adding spawn {:?} at room {:?}",
                rr_id,
                spawn_id,
                room_id
            );
        }
    }

    Ok(())
}

fn connect_rooms_to_entrance(
    rooms: &mut RoomRepository,
    entrance_id: ObjId,
    dir: Dir,
    rooms_grid: &LevelGrid,
    rooms_id: &Vec<RoomId>,
) -> Result<()> {
    let first_room_index = match dir {
        Dir::E => rooms_grid.get_index(0, 0),
        other => unimplemented!("for {:?}", other),
    };

    let first_room_id = rooms_id[first_room_index];
    rooms.add_portal(entrance_id, first_room_id, dir);
    Ok(())
}

fn create_rooms(
    objects: &mut Objects,
    rooms: &mut RoomRepository,
    labels: &mut Labels,
    grid: &LevelGrid,
) -> Result<Vec<ObjId>> {
    let mut ids = vec![];
    // create rooms
    for index in 0..grid.len() {
        let room_id = objects.create();
        let room = Room::new(room_id);
        rooms.add(room);

        let label = Label::new(room_id, format!("Random room {}", index).as_str());
        labels.add(label);

        ids.push(room_id);
    }

    // add portals
    trace!("adding portals to");
    trace!("{}", grid.print());

    for (a, b) in &grid.portals {
        let from_id = ids[*a];
        let to_id = ids[*b];
        let (x1, y1) = grid.get_coords(*a);
        let (x2, y2) = grid.get_coords(*b);

        let dir = if x2 > x1 {
            Dir::E
        } else if x2 < x1 {
            Dir::W
        } else if y2 > y1 {
            Dir::S
        } else if y2 < y1 {
            Dir::N
        } else {
            panic!("unexpected coords");
        };

        trace!(
            "{:?} ({},{}) to {:?} ({},{}) dir is {:?}",
            from_id,
            x1,
            y1,
            to_id,
            x2,
            y2,
            dir
        );

        rooms.add_portal(from_id, to_id, dir);
    }

    Ok(ids)
}
