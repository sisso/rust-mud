extern crate rand;

use rand::prelude::StdRng;
use rand::{thread_rng, Rng, RngCore, SeedableRng};
use std::collections::HashSet;

pub struct RandomLevelsCfg<'a> {
    pub rng: &'a mut StdRng,
    pub width: usize,
    pub height: usize,
    pub portal_prob: f32,
    pub deep_levels: u32,
}

pub struct RandomLevels {
    pub levels: Vec<LevelGrid>,
}

impl RandomLevels {
    pub fn new(cfg: &mut RandomLevelsCfg) -> Self {
        let mut levels = vec![];
        for deep in 0..cfg.deep_levels {
            let mut grid = LevelGrid::new(cfg);

            if deep < cfg.deep_levels - 1 {
                let down_index = cfg.rng.gen_range(0, grid.len());
                grid.down_portal = Some(down_index);
            }

            if deep > 0 {
                let up_index = cfg.rng.gen_range(0, grid.len());
                grid.up_portal = Some(up_index);
            }

            levels.push(grid);
        }

        RandomLevels { levels }
    }
}

pub struct LevelGrid {
    pub width: usize,
    pub height: usize,
    pub portals: HashSet<(usize, usize)>,
    pub up_portal: Option<usize>,
    pub down_portal: Option<usize>,
}

impl LevelGrid {
    pub fn is_portal(&self, room_a: usize, room_b: usize) -> bool {
        self.portals.contains(&(room_a, room_b)) || self.portals.contains(&(room_b, room_a))
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn len(&self) -> usize {
        self.width * self.height
    }

    pub fn get_coords(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.height)
    }

    pub fn neighbors(&self, index: usize) -> Vec<usize> {
        let mut list = vec![];
        let (x, y) = self.get_coords(index);

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

    pub fn new(cfg: &mut RandomLevelsCfg) -> LevelGrid {
        let mut rooms = LevelGrid {
            width: cfg.width,
            height: cfg.height,
            portals: Default::default(),
            up_portal: None,
            down_portal: None,
        };

        let door_prob = cfg.portal_prob;
        assert!(door_prob > 0.1);
        assert!(door_prob < 1.0);

        rooms.create_portals(cfg.rng, door_prob);
        rooms.connect_all_rooms();

        rooms
    }

    fn connect_all_rooms(&mut self) {
        // have sure that all rooms are reachable
        let mut visit_queue = vec![];
        visit_queue.push(0);
        let mut visited = HashSet::<usize>::new();
        'main: loop {
            if visit_queue.is_empty() {
                if visited.len() == self.len() {
                    // complete
                    break;
                } else {
                    // eprintln!("deadlock");

                    // deadlock, find any non visit room that is neighbor of an already visited
                    // and create a new portal

                    for index in 0..self.len() {
                        // skip already visited
                        if visited.contains(&index) {
                            continue;
                        }

                        for other_index in self.neighbors(index) {
                            if visited.contains(&other_index) {
                                // found a neighbor of already visited, create a portal
                                self.portals.insert((index, other_index));

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

                for other_index in self.neighbors(index) {
                    let valid =
                        !visited.contains(&other_index) && self.is_portal(index, other_index);
                    if valid {
                        // eprintln!("adding {}", other_index);
                        visit_queue.push(other_index);
                    }
                }
            }
        }
    }

    fn create_portals(&mut self, rng: &mut StdRng, door_prob: f32) {
        // for door each cell, there is 50% chance to have a door to N or W
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.get_index(x, y);

                if y != 0 && rng.gen::<f32>() < door_prob {
                    self.portals.insert((index, self.get_index(x, y - 1)));
                }

                if x != 0 && rng.gen::<f32>() < door_prob {
                    self.portals.insert((index, self.get_index(x - 1, y)));
                }
            }
        }
    }

    pub fn print(&self) -> String {
        /*
            .......
            .#-#.#.
            .|...|.
            .#-#-#.
            .......
        */
        let empty = ' ';
        let room = '#';
        let room_up = '^';
        let room_down = 'v';
        let room_up_down = 'X';
        let portal_v = '|';
        let portal_h = '-';

        let mut buffer = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let portal_n = if y == 0 {
                    false
                } else {
                    self.is_portal(self.get_index(x, y), self.get_index(x, y - 1))
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

            for x in 0..self.width {
                let index = y * self.width + x;

                let portal_w = if x == 0 {
                    false
                } else {
                    self.is_portal(self.get_index(x, y), self.get_index(x - 1, y))
                };

                if portal_w {
                    buffer.push(portal_h);
                } else {
                    buffer.push(empty);
                }

                let room_ch = match (
                    self.up_portal == Some(index),
                    self.down_portal == Some(index),
                ) {
                    (true, true) => room_up_down,
                    (true, false) => room_up,
                    (false, true) => room_down,
                    _ => room,
                };
                buffer.push(room_ch);
            }

            buffer.push(empty);
            buffer.push('\n');
        }

        for _x in 0..(self.width * 2 + 1) {
            buffer.push(empty);
        }

        buffer.push('\n');

        buffer
    }
}

pub struct RandomSpawnsCfg {
    spawns_to_add: Vec<u32>,
    seed: u64,
}

pub struct RandomSpawn {
    /// room index, x, y
    spawns: Vec<(u32, usize, usize)>,
}

impl RandomSpawn {
    pub fn new(cfg: RandomSpawnsCfg, rooms: &LevelGrid) -> Self {
        let mut rng: StdRng = SeedableRng::seed_from_u64(cfg.seed);
        let mut spawns: Vec<(u32, usize, usize)> = vec![];

        for spawn_id in cfg.spawns_to_add {
            loop {
                let candidate_x = rng.gen_range(0, rooms.width) as usize;
                let candidate_y = rng.gen_range(0, rooms.height) as usize;

                // check no collision
                if spawns
                    .iter()
                    .find(|(_, x, y)| *x == candidate_x && *y == candidate_y)
                    .is_some()
                {
                    continue;
                }

                spawns.push((spawn_id, candidate_x, candidate_y));
                break;
            }
        }

        RandomSpawn { spawns }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_generate_rooms() {
        let mut rng = SeedableRng::seed_from_u64(0);
        let grids = RandomLevels::new(&mut RandomLevelsCfg {
            rng: &mut rng,
            width: 5,
            height: 5,
            portal_prob: 0.5,
            deep_levels: 3,
        });

        for level in grids.levels {
            let buffer = level.print();
            println!("{}", buffer.as_str());
        }
    }
}
