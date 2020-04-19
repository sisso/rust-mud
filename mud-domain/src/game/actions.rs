use super::comm;
use super::container::Container;
use super::domain::*;
use super::mob::*;
use super::Outputs;
use crate::errors::Error::NotFoundFailure;
use crate::errors::{AsResult, Error, Result};
use crate::game::comm::{RoomMap, RoomMapCell};
use crate::game::item::ItemId;
use crate::game::loader::StaticId;
use crate::game::room::RoomRepository;
use crate::game::space_utils;
use commons::{ObjId, PlayerId};
use logs::*;
use std::collections::{HashMap, HashSet};
use std::process::id;

//#[derive(Debug, Clone)]
//pub enum Action {
//    Look,
//    Examine { target: ObjId },
//    MoveDir { dir: Dir },
//    Enter { target: ObjId },
//    Exit,
//    Rest,
//    Stand,
//    Equip { item: ItemId },
//    Remove { item: ItemId },
//    Pick { target: ObjId },
//    Kill { target: MobId },
//    Say { msg: String },
//    Move { target: ObjId },
//    Land { target: ObjId },
//    Launch,
//    Buy { target: StaticId },
//    Sell { target: ObjId },
//}
//
//// TODO: conflict with mob.MobAction
//#[derive(Debug, Clone)]
//pub struct MobAction_2 {
//    pub mob_id: MobId,
//    pub action: Action,
//}

pub fn look(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    outputs.private(mob_id, comm::look_description(container, mob_id)?);
    Ok(())
}

// TODO: do not allow to say empty
pub fn say(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    msg: &str,
) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let mob_label = container.labels.get(mob_id).as_result()?;
    let player_msg = comm::say_you_say(&msg);
    let room_msg = comm::say_someone_said(mob_label.label.as_str(), &msg);

    outputs.private(mob_id, player_msg);
    outputs.broadcast(Some(mob_id), room_id, room_msg);

    Ok(())
}

// optional PlayerId
pub fn mv(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    dir: Dir,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).as_result()?;
    let mob = container.mobs.get(mob_id).as_result()?;
    let room = container.rooms.get(location_id).as_result()?;
    let exit_room_id = room.get_exit(&dir);

    match exit_room_id {
        Some(exit_room_id) => {
            let previous_room_id = location_id;

            let mut mobs_to_move = vec![];
            mobs_to_move.push(mob_id);
            mobs_to_move.extend_from_slice(&mob.followers);

            for mob_id in mobs_to_move {
                if !container.objects.exists(mob_id) {
                    warn!("follower {:?} do not exists!", mob_id);
                    continue;
                }

                // change mob place
                container.locations.set(mob_id, exit_room_id);

                let mob_label = container.labels.get_label_f(mob_id);

                // TODO: maybe exclude output for people in the same group?
                let look = comm::look_description(&container, mob_id).unwrap();
                let privte_msg = format!("{}\n\n{}", comm::move_you_move(&dir), look);
                let enter_room_msg = comm::move_come(mob_label, &dir.inv());
                let exit_room_msg = comm::move_goes(mob_label, &dir);

                outputs.private(mob_id, privte_msg);
                outputs.broadcast(Some(mob_id), previous_room_id, exit_room_msg);
                outputs.broadcast(Some(mob_id), exit_room_id, enter_room_msg);
            }

            Ok(())
        }
        None => {
            outputs.private(mob_id, comm::move_not_possible(&dir));
            Err(Error::InvalidArgumentFailure)
        }
    }
}

// optional PlayerId
pub fn attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_mob_id: MobId,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).as_result()?;
    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let target_label = container.labels.get_label(target_mob_id).as_result()?;

    let player_msg = comm::attack_player_initiate(target_label);
    let room_msg = comm::attack_mob_initiate_attack(mob_label, target_label);

    outputs.private(mob_id, player_msg);
    outputs.broadcast(Some(mob_id), location_id, room_msg);

    container.mobs.set_mob_attack_target(mob_id, target_mob_id);

    Ok(())
}

// optional PlayerId
pub fn rest(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let mob = container.mobs.get(mob_id).as_result()?;

    let total_time = container.time.total;

    if mob.is_combat() {
        outputs.private(mob_id, comm::rest_fail_in_combat());
        return Err(Error::InvalidStateFailure);
    }

    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(mob_id, comm::rest_start());
    outputs.broadcast(Some(mob_id), room_id, comm::rest_start_others(mob_label));

    container.mobs.update(mob_id, |mob| {
        let _ = mob.set_action_rest(total_time);
    })
}

// optional PlayerId
pub fn stand(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let ctx = container.get_mob_ctx(mob_id).as_result()?;

    if ctx.mob.is_resting() {
        outputs.private(mob_id, comm::stand_fail_not_resting());
        return Err(Error::InvalidStateFailure);
    }

    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(mob_id, comm::stand_up());
    outputs.broadcast(Some(mob_id), ctx.room.id, comm::stand_up_others(mob_label));
    container.mobs.update(mob_id, |mob| {
        let _ = mob.stop_rest();
    });

    Ok(())
}

pub fn enter(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    arguments: &str,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).as_result()?;
    let candidates = space_utils::find_ships_at(container, location_id);
    let target = container
        .labels
        .search_codes(&candidates, arguments)
        .first()
        .cloned();

    trace!(
        "mob_id: {:?} at {:?}, candidates: {:?}, target: {:?}",
        mob_id,
        location_id,
        candidates,
        target
    );

    match target {
        Some(target) => enter_do(container, outputs, mob_id, target),

        None if arguments.is_empty() => {
            let codes = container.labels.resolve_codes(&candidates);
            outputs.private(mob_id, comm::enter_list(&codes));
            Err(Error::InvalidArgumentFailure)
        }

        None => {
            let codes = container.labels.resolve_codes(&candidates);
            outputs.private(mob_id, comm::enter_invalid(arguments, &codes));
            Err(Error::InvalidArgumentFailure)
        }
    }
}

pub fn enter_do(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: ObjId,
) -> Result<()> {
    let current_location = container.locations.get(mob_id).as_result()?;

    // find target room
    let candidate = space_utils::find_children_rooms_with_can_exit(container, target_id)
        .first()
        .cloned();

    match candidate {
        Some(location_id) => {
            let target_label = container.labels.get_label_f(target_id);
            let mob_label = container.labels.get_label_f(mob_id);

            // change mob place
            container.locations.set(mob_id, location_id);

            // emmit messages
            outputs.private(mob_id, comm::enter_player(target_label));
            outputs.private(mob_id, comm::look_description(&container, mob_id).unwrap());
            outputs.broadcast(
                Some(mob_id),
                current_location,
                comm::enter_others(mob_label, target_label),
            );
            outputs.broadcast(
                Some(mob_id),
                location_id,
                comm::enter_others_other_side(mob_label),
            );

            Ok(())
        }

        None => {
            outputs.private(mob_id, comm::enter_fail());
            Err(Error::InvalidArgumentFailure)
        }
    }
}

pub fn out(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let location_id = container.locations.get(mob_id).as_result()?;

    let can_exit = container.rooms.get(location_id).as_result()?.can_exit;

    if !can_exit {
        outputs.private(mob_id, comm::out_fail());
        return Err(Error::InvalidArgumentFailure);
    }

    let parents = container.locations.list_parents(location_id);
    let from_id = parents.get(0).cloned().as_result()?;
    let target_id = parents
        .iter()
        .filter(|&&id| container.rooms.exists(id))
        .next()
        .cloned();

    if let Some(target_id) = target_id {
        let from_label = container.labels.get_label_f(from_id);
        let mob_label = container.labels.get_label_f(mob_id);

        // change mob place
        container.locations.set(mob_id, target_id);

        // emmit messages
        outputs.private(mob_id, comm::out_player());
        outputs.private(mob_id, comm::look_description(&container, mob_id).unwrap());
        outputs.broadcast(Some(mob_id), location_id, comm::out_others(mob_label));
        outputs.broadcast(
            Some(mob_id),
            target_id,
            comm::out_others_other_side(mob_label, from_label),
        );

        Ok(())
    } else {
        outputs.private(mob_id, comm::out_fail_bad_outside());
        Err(Error::InvalidArgumentFailure)
    }
}

pub fn show_map(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let location_id = container.locations.get(mob_id).as_result()?;
    let room_map = generate_room_maps(location_id, 3, &container.rooms)?;

    let mut labels = HashMap::new();

    for cell in &room_map.cells {
        if let RoomMapCell::Room(obj_id) = cell {
            let label = container.labels.get_label_f(*obj_id).to_string();
            labels.insert(*obj_id, label);
        }
    }

    outputs.private(mob_id, comm::print_room_map(location_id, room_map, &labels));
    Ok(())
}

fn generate_room_maps(
    location_id: ObjId,
    max_distance: u32,
    rooms: &RoomRepository,
) -> Result<RoomMap> {
    let mut visited = HashMap::<ObjId, (i32, i32)>::new();
    let mut portals = HashSet::new();

    load_rooms_into_coords_map(location_id, max_distance, rooms, &mut visited, &mut portals)?;
    room_map_from_rooms_coords(visited, portals)
}

fn room_map_from_rooms_coords(
    mut visited: HashMap<ObjId, (i32, i32)>,
    portals: HashSet<(ObjId, ObjId)>,
) -> Result<RoomMap> {
    // normalize in the top left corner
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    for (_id, (x, y)) in &visited {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    // normalize
    for (_id, (x, y)) in &mut visited {
        *x -= min_x;
        *y -= min_y;
    }
    // compute cells size
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    // send to array
    let mut cells = vec![];
    // trace!("min {},{} max {},{} width {} height {}", min_x, min_y, max_x, max_y, width, height);

    let rooms_by_coords = visited
        .iter()
        .map(|(id, (x, y))| ((*x, *y), *id))
        .collect::<HashMap<_, _>>();

    let get_room = |x, y| -> Option<ObjId> { rooms_by_coords.get(&(x, y)).cloned() };

    let is_portal = |x0, y0, x1, y1| -> bool {
        let id0 = get_room(x0, y0);
        let id1 = get_room(x1, y1);

        let (id0, id1) = match (id0, id1) {
            (Some(id0), Some(id1)) => (id0, id1),
            _ => return false,
        };

        portals.contains(&(id0, id1))
    };

    for y in 0..height {
        if y != 0 {
            for x in 0..width {
                if is_portal(x, y, x, y - 1) {
                    cells.push(RoomMapCell::DoorVer);
                } else {
                    cells.push(RoomMapCell::Empty);
                }

                if x != width - 1 {
                    cells.push(RoomMapCell::Empty);
                }
            }
        }

        for x in 0..width {
            if x != 0 {
                if is_portal(x, y, x - 1, y) {
                    cells.push(RoomMapCell::DoorHor);
                } else {
                    cells.push(RoomMapCell::Empty);
                }
            }

            if let Some(room_id) = get_room(x, y) {
                cells.push(RoomMapCell::Room(room_id));
            } else {
                cells.push(RoomMapCell::Empty);
            }
        }
    }

    // trace!("cells {:?}", cells);
    Ok(RoomMap {
        width: (width * 2 - 1) as u32,
        height: (height * 2 - 1) as u32,
        cells: cells,
    })
}

fn load_rooms_into_coords_map(
    location_id: ObjId,
    max_distance: u32,
    rooms: &RoomRepository,
    visited: &mut HashMap<ObjId, (i32, i32)>,
    portals: &mut HashSet<(ObjId, ObjId)>,
) -> Result<()> {
    let mut queue = vec![];
    queue.push((location_id, 0, 0));
    loop {
        let (id, x, y) = match queue.pop() {
            Some(value) => value,
            _ => break,
        };

        match visited.get(&id) {
            // new value is lower that already existent tone
            Some((x1, y1)) if x1 + y1 > x + y => {
                // trace!("{:?} replace {},{} by {},{}", id, x1, y1, x, y);
                visited.insert(id, (x, y));
                continue;
            }
            // skip already vistied
            Some(_) => continue,
            None => {}
        };

        // trace!("{:?} adding at {},{}", id, x, y);
        visited.insert(id, (x, y));

        for (dir, target_id) in rooms.get_portals(id)? {
            let (tx, ty) = match dir {
                Dir::N => (x, y - 1),
                Dir::S => (x, y + 1),
                Dir::E => (x + 1, y),
                Dir::W => (x - 1, y),
            };

            let tx: i32 = tx;
            let ty: i32 = ty;

            portals.insert((id, *target_id));
            portals.insert((*target_id, id));

            if tx.abs() as u32 > max_distance || ty.abs() as u32 > max_distance {
                continue;
            }

            queue.push((*target_id, tx, ty));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::game::comm::RoomMapCell;
    use crate::game::domain::Dir;
    use crate::game::room::{Room, RoomRepository};
    use commons::ObjId;

    #[test]
    fn test_generate_room_maps() {
        /*
            0 1-2
            | | |
            3-4-5
        */

        let mut rooms = RoomRepository::new();
        rooms.add(Room::new(ObjId(0)));
        rooms.add(Room::new(ObjId(1)));
        rooms.add(Room::new(ObjId(2)));
        rooms.add(Room::new(ObjId(3)));
        rooms.add(Room::new(ObjId(4)));
        rooms.add(Room::new(ObjId(5)));

        rooms.add_portal(0.into(), 3.into(), Dir::S);
        rooms.add_portal(1.into(), 2.into(), Dir::E);
        rooms.add_portal(1.into(), 4.into(), Dir::S);
        rooms.add_portal(2.into(), 5.into(), Dir::S);
        rooms.add_portal(3.into(), 4.into(), Dir::E);
        rooms.add_portal(4.into(), 5.into(), Dir::E);

        // 0 1
        // | |
        // 3-4
        let room_map = super::generate_room_maps(0.into(), 1, &rooms).unwrap();

        assert_eq!(room_map.width, 3);
        assert_eq!(room_map.height, 3);

        let expected = vec![
            RoomMapCell::Room(0.into()),
            RoomMapCell::Empty,
            RoomMapCell::Room(1.into()),
            RoomMapCell::DoorVer,
            RoomMapCell::Empty,
            RoomMapCell::DoorVer,
            RoomMapCell::Room(3.into()),
            RoomMapCell::DoorHor,
            RoomMapCell::Room(4.into()),
        ];

        assert_eq!(room_map.cells, expected);

        // 0 1-2
        // | | |
        // 3-4-5
        let room_map = super::generate_room_maps(0.into(), 2, &rooms).unwrap();

        assert_eq!(room_map.width, 5);
        assert_eq!(room_map.height, 3);

        let expected = vec![
            RoomMapCell::Room(0.into()),
            RoomMapCell::Empty,
            RoomMapCell::Room(1.into()),
            RoomMapCell::DoorHor,
            RoomMapCell::Room(2.into()),
            RoomMapCell::DoorVer,
            RoomMapCell::Empty,
            RoomMapCell::DoorVer,
            RoomMapCell::Empty,
            RoomMapCell::DoorVer,
            RoomMapCell::Room(3.into()),
            RoomMapCell::DoorHor,
            RoomMapCell::Room(4.into()),
            RoomMapCell::DoorHor,
            RoomMapCell::Room(5.into()),
        ];

        assert_eq!(room_map.cells, expected);
    }
}
