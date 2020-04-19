extern crate rand;

use std::collections::{HashMap};
use commons::{ObjId};
use mud_domain::game::domain::Dir;

trait Rooms {
    fn portals(&self, id: ObjId) -> Vec<(Dir, ObjId)>;
}

#[derive(Debug, Clone)]
struct RoomsMap {
    width: usize,
    height: usize,
    list: Vec<Option<ObjId>>,
}

// TODO: conflicts?
fn generate_map(initial: ObjId, max_distance: u32, rooms: &dyn Rooms) -> RoomsMap {

    let mut queue = vec![];
    queue.push((initial, 0, 0));

    let mut visited = HashMap::<ObjId, (i32, i32)>::new();

    loop {
        let (id, x, y) = match queue.pop() {
            Some(value) => value,
            _ => break,
        };

        match visited.get(&id) {
            // new value is lower that already existent tone
            Some((x1, y1)) if x1 + y1 > x + y => {
                print!("replace {},{} by {},{} for {:?}", x1, y1, x, y, id);
                visited.insert(id, (x, y));
                continue;
            },
            // skip already vistied
            Some(_) => continue,
            None => {}
        };

        visited.insert(id, (x, y));

        // if let Some(old) = map.id_by_coord.insert((x, y), id) {
        //     println!("TODO conflict at {},{} between {:?} and {:?}", x, y, id, old);
        // }
        //
        for (dir, target_id) in rooms.portals(id) {
            let (tx,ty) = match dir {
                Dir::N => (x, y - 1),
                Dir::S => (x, y + 1),
                Dir::E => (x + 1, y),
                Dir::W => (x - 1, y),
            };

            let tx: i32 = tx;
            let ty: i32 = ty;

            if tx.abs() as u32 >= max_distance || ty.abs() as u32 >= max_distance {
                continue;
            }

            queue.push((target_id, tx, ty));
        }
    }

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

    let mut width = max_x - min_x;
    let mut height = max_y - min_y;
    // println!("min {},{} max {},{} width {} height {}", min_x, min_y, max_x, max_y, width, height);

    // normalize
    for (_id, (x, y)) in &mut visited {
        *x -= min_x;
        *y -= min_y;
    }

    // send to array
    let mut list = vec![];
    for (id, (x, y)) in &visited {
        // println!("{:?} {},{}", id, x, y);
        let index = *x + *y * (height as i32);

        for _ in (list.len() as i32)..(index - 1) {
            list.push(None);
        }

        list.push(Some(*id));
    }

    RoomsMap {
        width: width as usize,
        height: height as usize,
        list
    }
}

fn main() {
    struct RoomsImpl;

    /*
        0 1-2
        | | |
        3-4-5

        w: 6
        h: 3


    */
    impl Rooms for RoomsImpl {
        fn portals(&self, id: ObjId) -> Vec<(Dir, ObjId)> {
            match id {
                ObjId(0) => vec![(Dir::S, ObjId(3))],
                ObjId(1) => vec![(Dir::E, ObjId(2)), (Dir::S, ObjId(4))],
                ObjId(2) => vec![(Dir::W, ObjId(1)), (Dir::S, ObjId(5))],
                ObjId(3) => vec![(Dir::N, ObjId(0)), (Dir::E, ObjId(4))],
                ObjId(4) => vec![(Dir::W, ObjId(3)), (Dir::E, ObjId(5)), (Dir::N, ObjId(1))],
                ObjId(5) => vec![(Dir::W, ObjId(4)), (Dir::N, ObjId(2))],
                _ => panic!("unexpected id {:?}", id),
            }
        }
    };

    let rooms = RoomsImpl {};
    let map = generate_map(ObjId(0), 10, &rooms);
    println!("{:?}", map);
}
