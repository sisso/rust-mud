use crate::game::system::SystemCtx;
use crate::game::random_rooms_generator::{RoomGrid, RoomGridCfg};
use crate::game::random_rooms::RandomRoomsCfg;
use crate::game::room::{RoomRepository, Room};
use logs::*;
use crate::game::labels::{Labels, Label};
use crate::game::obj::Objects;
use crate::game::domain::Dir;
use crate::errors::Result;
use crate::game::container::Container;
use commons::ObjId;

pub fn run(ctx: &mut SystemCtx) {

}

pub fn init(container: &mut Container) {
    let random_rooms_repo = &mut container.random_rooms;
    let objects = &mut container.objects;
    let rooms = &mut container.rooms;
    let labels = &mut container.labels;

    for rr in random_rooms_repo.list_states_mut() {
        if rr.generated {
            continue;
        }

        info!("{:?} generating random rooms", rr.cfg.id);

        let cfg = RoomGridCfg {
            seed: Some(rr.cfg.seed),
            width: rr.cfg.width as usize,
            height: rr.cfg.height as usize,
            portal_prob: None
        };

        let rooms_grid = RoomGrid::new(cfg);

        let rooms_ids = match create_rooms(objects, rooms, labels,  &rooms_grid) {
            Err(err) => {
                warn!("{:?} error when generating rooms from grid {:?}", rr.cfg.id, err);
                continue;
            },
            Ok(ids) => ids,
        };

        connect_rooms_to_entrance(rooms, rr.cfg.entrance_id, rr.cfg.entrance_dir, &rooms_grid, &rooms_ids);

        rr.generated = true;
    }
}

fn connect_rooms_to_entrance(rooms: &mut RoomRepository, entrance_id: ObjId, dir: Dir, rooms_grid: &RoomGrid, ids: &Vec<ObjId>) -> Result<()> {
    let first_room_index = match dir {
        Dir::E => {
           rooms_grid.get_index(0, 0)
        },
        other => unimplemented!("for {:?}", other),
    };

    let first_room_id = ids[first_room_index];
    rooms.add_portal(entrance_id, first_room_id, dir);
    Ok(())
}

fn create_rooms(objects: &mut Objects, rooms: &mut RoomRepository, labels: &mut Labels, grid: &RoomGrid) -> Result<Vec<ObjId>> {
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

    for (a,b) in &grid.portals {
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

        trace!("{:?} ({},{}) to {:?} ({},{}) dir is {:?}", from_id, x1, y1, to_id, x2, y2, dir);

        rooms.add_portal(from_id, to_id, dir);
    }

    Ok(ids)
}
