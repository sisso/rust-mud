extern crate rand;

use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::process::id;

pub struct Rooms {
    width: usize,
    height: usize,
    portals: HashSet<(usize, usize)>,
}

impl Rooms {
    pub fn is_portal(&self, room_a: usize, room_b: usize) -> bool {
        self.portals.contains(&(room_a, room_b)) || self.portals.contains(&(room_b, room_a))
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn len(&self) -> usize {
        self.width * self.height
    }

    pub fn coords(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.height)
    }

    fn neighbors(&self, index: usize) -> Vec<usize> {
        let mut list = vec![];
        let (x, y) = self.coords(index);

        if x > 0 {
            list.push(index - 1);
        }

        if x < self.width - 1 {
            list.push(index + 1);
        }

        if y > 0 {
            list.push(index - self.width);
        }

        if y < self.height - 1 {
            list.push(index + self.width);
        }

        list
    }
}

fn generate(seed: u64, width: usize, height: usize, door_prob: f32) -> Rooms {
    let mut rooms = Rooms {
        width,
        height,
        portals: Default::default(),
    };

    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    create_rooms_and_portals(&mut rng, &mut rooms, door_prob);
    make_full_connected(&mut rooms);

    rooms
}

fn make_full_connected(rooms: &mut Rooms) {
    // have sure that all rooms are reachable
    let mut visit_queue = vec![];
    visit_queue.push(0);
    let mut visited = HashSet::<usize>::new();
    'main: loop {
        if visit_queue.is_empty() {
            if visited.len() == rooms.len() {
                // complete
                break;
            } else {
                // eprintln!("deadlock");

                // deadlock, find any non visit room that is neighbor of an already visited
                // and create a new portal

                for index in 0..rooms.len() {
                    // skip already visited
                    if visited.contains(&index) {
                        continue;
                    }

                    for other_index in rooms.neighbors(index) {
                        if visited.contains(&other_index) {
                            // found a neighbor of already visited, create a portal
                            rooms.portals.insert((index, other_index));

                            // add current to be vistied
                            visit_queue.push(index);

                            // eprintln!("adding portal between {} and {}", index, other_index);

                            continue 'main;
                        }
                    }
                }
            }
        } else {
            let index = visit_queue.pop().unwrap();
            visited.insert(index);

            // eprintln!("current {}", index);

            for other_index in rooms.neighbors(index) {
                let valid = !visited.contains(&other_index) && rooms.is_portal(index, other_index);
                if valid {
                    // eprintln!("adding {}", other_index);
                    visit_queue.push(other_index);
                }
            }
        }
    }
}

fn create_rooms_and_portals(rng: &mut StdRng, rooms: &mut Rooms, door_prob: f32) {
    // for door each cell, there is 50% chance to have a door to N or W
    for y in 0..rooms.height {
        for x in 0..rooms.width {
            let index = rooms.get_index(x, y);

            if y != 0 && rng.gen::<f32>() < door_prob {
                rooms.portals.insert((index, rooms.get_index(x, y - 1)));
            }

            if x != 0 && rng.gen::<f32>() < door_prob {
                rooms.portals.insert((index, rooms.get_index(x - 1, y)));
            }
        }
    }
}

fn print(map: &Rooms) {
    /*
        .......
        .#-#.#.
        .|...|.
        .#-#-#.
        .......
    */
    let empty = ' ';
    let room = '#';
    let portal_v = '|';
    let portal_h = '-';

    let mut buffer = String::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let portal_n = if y == 0 {
                false
            } else {
                map.is_portal(map.get_index(x, y), map.get_index(x, y - 1))
            };

            buffer.push(empty);
            if portal_n {
                buffer.push(portal_v);
            } else {
                buffer.push(empty);
            }
        }

        buffer.push(empty);
        buffer.push('\n');

        for x in 0..map.width {
            let portal_w = if x == 0 {
                false
            } else {
                map.is_portal(map.get_index(x, y), map.get_index(x - 1, y))
            };

            if portal_w {
                buffer.push(portal_h);
            } else {
                buffer.push(empty);
            }
            buffer.push(room);
        }

        buffer.push(empty);
        buffer.push('\n');
    }

    for x in 0..(map.width * 2 + 1) {
        buffer.push(empty);
    }

    buffer.push('\n');
    println!("{}", buffer);
}

fn main() {
    let mut buffer = String::new();
    let mut seed = 0;
    loop {
        std::io::stdin().read_line(&mut buffer).unwrap();
        let rooms = generate(seed, 5, 5, 0.60);
        print(&rooms);

        seed += 1;
    }

    // for (a, b) in rooms.portals {
    //     println!("{} {}", a, b);
    // }
}
