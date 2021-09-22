// use mud_domain::random_grid;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

type GlobalCoord = [i32; 2];
type SectorCoord = [i32; 2];
type LocalCoord = [i32; 2];

pub enum Dir {
    N,
    E,
    S,
    W,
}

#[derive(Debug)]
pub struct Doors {
    pub n: bool,
    pub e: bool,
    pub w: bool,
    pub s: bool,
}

impl Doors {
    pub fn as_array(&self) -> [bool; 4] {
        [self.n, self.e, self.s, self.w]
    }
}

#[derive(Debug)]
pub struct Cell {
    pub coords: GlobalCoord,
    pub doors: Doors,
}

#[derive(Debug)]
pub struct Sector {
    grid: commons::grid::Grid<Cell>,
}

impl Sector {
    pub fn get(&self, coord: &LocalCoord) -> Option<&Cell> {
        self.cells.get(coord)
    }
}

#[derive(Debug)]
pub struct Cfg {
    seed: u64,
    portal_prob: f32,
    sector_size: i32,
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg {
            seed: 0,
            portal_prob: 0.5,
            sector_size: 5,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    cfg: Cfg,
    sectors: HashMap<SectorCoord, Sector>,
}

impl Grid {}

impl Grid {
    pub fn new(cfg: Cfg) -> Self {
        Grid {
            cfg: cfg,
            sectors: Default::default(),
        }
    }

    pub fn get(&self, coord: &GlobalCoord) -> Option<&Cell> {
        let (sector_coord, local_coord) = self.get_sector_coord(coord);
        self.sectors
            .get(&sector_coord)
            .and_then(|sector| sector.get(&local_coord))
    }

    pub fn create(&mut self, coord: &GlobalCoord) -> &Cell {
        let (sector_coord, local_coord) = self.get_sector_coord(coord);

        let sector = self.sectors.entry(sector_coord).or_insert_with(|| Sector {
            cells: Default::default(),
        });

        let cell = sector.cells.entry(local_coord).or_insert_with(|| Cell {
            coords: coord.clone(),
            doors: Doors {
                n: false,
                e: false,
                w: false,
                s: false,
            },
        });

        cell
    }

    fn get_sector(&self, coord: &GlobalCoord) -> Option<&Sector> {
        self.sectors.get(coord)
    }

    fn new_sector(&self, coords: &SectorCoord) -> Sector {
        // let cfg = random_grid::RandomGridCfg {
        //     width: self.cfg.sector_size as usize,
        //     height: self.cfg.sector_size as usize,
        //     portal_prob: self.cfg.portal_prob,
        //     deep_levels: 0,
        // };
        //
        // let mut rng: StdRng = SeedableRng::seed_from_u64(self.get_sector_seed(coords));
        // let grid = random_grid::LevelGrid::new(&cfg, &mut rng);
        //
        // let mut cells = vec![];
        //
        // for i in 0..grid.len() {
        //     let (x,y) = grid.get_coords(i);
        //     let neighbors = grid.neighbors_connected(i);
        //
        //
        // }
        //
        // Sector {
        //     cells
        // }

        // 1. for each border generate 1..n contact points
        // 2. add rooms with contact points to the grid
        // 3. generate random internal rooms as a finite graph
        let borders = commons::grid::get_4_neighbours(coords.into());
        let borders_rooms = borders
            .iter()
            .map(|sector| self.get_border_connection_cells(coords, &sector.as_array()))
            .collect();

        todo!()
    }

    fn get_sector_connection_cells(
        &self,
        coords_a: &SectorCoord,
        coords_b: &SectorCoord,
    ) -> Vec<i32> {
        commons::grid::get_4_neighbours(&coords_a.into())
            .into_iter()
            .zip(commons::grid::DIR_ALL)
            .map(|(sector, dir)| {
                let cells = self.get_border_connection_cells(coords, &sector.as_array());
                match dir {
                    commons::grid::Dir::N => {}
                    commons::grid::Dir::E => {}
                    commons::grid::Dir::S => {}
                    commons::grid::Dir::W => {}
                }
            })
            .collect()
    }

    fn get_border_connection_cells(
        &self,
        coords_a: &SectorCoord,
        coords_b: &SectorCoord,
    ) -> Vec<i32> {
        let seed = self.get_sectors_connection_seed(coords_a, coords_b);
        let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

        let mut candidates: Vec<i32> = (0..self.cfg.sector_size).collect();
        candidates.shuffle(&mut rng);

        candidates
            .into_iter()
            .take(rng.gen_range(1..self.cfg.sector_size) as usize)
            .collect()
    }

    fn get_sectors_connection_seed(&self, coords_a: &SectorCoord, coords_b: &SectorCoord) -> u64 {
        let mut h = DefaultHasher::default();

        self.cfg.seed.hash(&mut h);

        if coords_a[0] > coords_b[0] {
            coords_a[0].hash(&mut h);
            coords_b[0].hash(&mut h);
        } else {
            coords_b[0].hash(&mut h);
            coords_a[0].hash(&mut h);
        }

        if coords_a[1] > coords_b[1] {
            coords_a[1].hash(&mut h);
            coords_b[1].hash(&mut h);
        } else {
            coords_b[1].hash(&mut h);
            coords_a[1].hash(&mut h);
        }

        h.finish()
    }

    fn get_sector_seed(&self, coords: &SectorCoord) -> u64 {
        let mut h = DefaultHasher::default();
        self.cfg.seed.hash(&mut h);
        coords[0].hash(&mut h);
        coords[1].hash(&mut h);
        h.finish()
    }

    // s = 3
    // -1, 0 = (-1, 0) (2, 0)
    // -2, 0 = (-1, 0) (1, 0)
    // -3, 0 = (-1, 0) (0, 0)
    // -4, 0 = (-2, 0) (2, 0)
    pub fn get_sector_coord(&self, coord: &GlobalCoord) -> (SectorCoord, LocalCoord) {
        fn compute(s: i32, v: i32) -> (i32, i32) {
            if v >= 0 {
                (v / s, v % s)
            } else {
                let sector = ((v + 1) / s) - 1;
                // size 5, v = -1: (5) + (-1 % 5) = 5 - 1 = 4
                // size 5, v = -2: (5) + (-2 % 5) = 5 - 2 = 3
                // size 5, v = -3: (5) + (-3 % 5) = 5 - 3 = 2
                // size 5, v = -4: (5) + (-4 % 5) = 5 - 4 = 1
                // size 5, v = -5: (5) + (-5 % 5) = 5 - 0 = 5
                // size 5, v = -6: (5) + (-6 % 5) = 5 - 1 = 4
                let mut cell = (v % s);
                if cell != 0 {
                    cell = s + cell;
                }
                (sector, cell)
            }
        }

        let (sectorx, cellx) = compute(self.cfg.sector_size, coord[0]);
        let (sectory, celly) = compute(self.cfg.sector_size, coord[1]);

        ([sectorx, sectory], [cellx, celly])
    }
}

fn main() {
    let mut g = Grid::new(Cfg::default());

    let room = g.get(&[0, 0]);
    print!("#");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sector_coord() {
        let size = 5;
        let mut g = Grid::new(Cfg {
            seed: 0,
            portal_prob: 0.5,
            sector_size: size,
        });
        let last_i = size - 1;

        // first sector
        assert_eq!(([0, 0], [0, 0]), g.get_sector_coord(&[0, 0]));
        assert_eq!(([0, 0], [1, 0]), g.get_sector_coord(&[1, 0]));
        assert_eq!(([0, 0], [3, 2]), g.get_sector_coord(&[3, 2]));
        // next sector
        assert_eq!(([1, 0], [0, last_i]), g.get_sector_coord(&[size, last_i]));
        // bottom down sector
        assert_eq!(([1, 1], [0, 0]), g.get_sector_coord(&[size, size]));
        // negative
        assert_eq!(([-1, 0], [last_i, 0]), g.get_sector_coord(&[-1, 0]));
        assert_eq!(([0, -1], [0, last_i]), g.get_sector_coord(&[0, -1]));
        assert_eq!(([0, -1], [0, last_i]), g.get_sector_coord(&[0, -1]));
        assert_eq!(([-1, -1], [last_i, last_i]), g.get_sector_coord(&[-1, -1]));
        assert_eq!(([-1, 0], [0, 0]), g.get_sector_coord(&[-size, 0]));
        assert_eq!(([-2, 0], [last_i, 0]), g.get_sector_coord(&[-size - 1, 0]));
    }

    #[test]
    fn test_empty() {
        let mut g = Grid::new(Cfg::default());
        let room = g.get(&[0, 0]);
        assert!(room.is_none());

        let room = g.create(&[0, 0]);
        assert_eq!(0, room.coords[0]);
        assert_eq!(0, room.coords[1]);

        assert!(room.doors.as_array().iter().filter(|i| **i).count() > 0);
    }

    #[test]
    fn test_get_sectors_connection_seed() {
        let g = Grid::new(Cfg::default());
        let a = [41, -53];
        let b = [41, -52];

        let h1 = g.get_sectors_connection_seed(&a, &b);
        let h2 = g.get_sectors_connection_seed(&b, &a);
        assert_eq!(h1, h2);
        assert_ne!(0, h1);

        let c = [40, -53];
        let h3 = g.get_sectors_connection_seed(&a, &c);
        assert_ne!(h1, h3);

        let d = [-41, 53];
        let e = [-41, 52];
        let h4 = g.get_sectors_connection_seed(&d, &e);
        assert_ne!(h1, h4);
    }

    #[test]
    fn test_get_sector_connections() {
        let g = Grid::new(Cfg::default());

        let a = [41, -53];
        let b = [41, -52];

        let connections_a = g.get_border_connection_cells(&a, &b);
        let connections_b = g.get_border_connection_cells(&b, &a);

        assert_eq!(connections_a, connections_b);
        assert!(connections_a.len() > 0);
    }
}
