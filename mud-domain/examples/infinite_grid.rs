// use mud_domain::random_grid;
use commons::{grid, V2I};
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

type GlobalCoord = commons::V2I;
type SectorCoord = commons::V2I;
type LocalCoord = commons::V2I;

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
    // pub coords: GlobalCoord,
// pub doors: Doors,
}

#[derive(Debug)]
pub struct Sector {
    grid: commons::grid::Grid<Cell>,
}

impl Sector {
    pub fn get(&self, coord: &LocalCoord) -> Option<&Cell> {
        self.grid.get_at(coord)
    }
}

#[derive(Debug)]
pub struct Cfg {
    seed: u64,
    portal_prob: f32,
    sector_size: u32,
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
pub struct InfiniteGrid {
    cfg: Cfg,
    sectors: HashMap<SectorCoord, Sector>,
    portals: HashSet<(SectorCoord, SectorCoord)>,
}

impl InfiniteGrid {}

impl InfiniteGrid {
    pub fn new(cfg: Cfg) -> Self {
        InfiniteGrid {
            cfg: cfg,
            sectors: Default::default(),
            portals: Default::default(),
        }
    }

    pub fn get(&self, coord: &GlobalCoord) -> Option<&Cell> {
        let (sector_coord, local_coord) = self.get_sector_coord(coord);
        self.sectors
            .get(&sector_coord)
            .and_then(|sector| sector.get(&local_coord))
    }

    pub fn create(&mut self, coord: &GlobalCoord) {
        let (sector_coord, local_coord) = self.get_sector_coord(coord);

        // let sector = self.sectors.entry(sector_coord).or_insert_with(|| Sector {
        //     grid: commons::grid::Grid::new(self.cfg.sector_size, self.cfg.sector_size),
        // });

        // let cell = sector.grid.entry(local_coord).or_insert_with(|| Cell {
        //     coords: coord.clone(),
        //     doors: Doors {
        //         n: false,
        //         e: false,
        //         w: false,
        //         s: false,
        //     },
        // });
        todo!()
    }

    fn get_sector(&self, coord: &GlobalCoord) -> Option<&Sector> {
        self.sectors.get(coord)
    }

    fn new_sector(&self, sector_coords: &SectorCoord) -> Sector {
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
        // let borders = commons::grid::get_4_neighbours(coords.into());
        // let borders_rooms = borders
        //     .iter()
        //     .map(|sector| self.get_border_connection_cells(coords, &sector))
        //     .collect();

        let mut grid = grid::Grid::<Cell>::new_square(self.cfg.sector_size);
        let borders = self.get_sector_connection_cells(sector_coords);
        for cell in borders {
            grid.set_at(&cell, Some(Cell {}))
        }

        todo!()
    }

    fn get_sector_connection_cells(&self, sector_cords: &SectorCoord) -> Vec<V2I> {
        commons::grid::get_4_neighbours(&sector_cords)
            .into_iter()
            .flat_map(|(dir, sector)| {
                let cells = self.get_border_connection_cells(sector_cords, &sector);
                let result: Vec<_> = match dir {
                    commons::grid::Dir::N => {
                        cells.into_iter().map(|index| V2I::new(index, 0)).collect()
                    }
                    commons::grid::Dir::E => cells
                        .into_iter()
                        .map(|index| V2I::new(self.cfg.sector_size as i32 - 1, index))
                        .collect(),
                    commons::grid::Dir::S => cells
                        .into_iter()
                        .map(|index| V2I::new(index, self.cfg.sector_size as i32 - 1))
                        .collect(),
                    commons::grid::Dir::W => {
                        cells.into_iter().map(|index| V2I::new(0, index)).collect()
                    }
                };
                result
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

        let mut candidates: Vec<u32> = (0..self.cfg.sector_size).collect();
        candidates.shuffle(&mut rng);

        candidates
            .into_iter()
            .take(rng.gen_range(1..self.cfg.sector_size) as usize)
            .map(|i| i as i32)
            .collect()
    }

    fn get_sectors_connection_seed(&self, coords_a: &SectorCoord, coords_b: &SectorCoord) -> u64 {
        let mut h = DefaultHasher::default();

        self.cfg.seed.hash(&mut h);

        if coords_a.x > coords_b.x {
            coords_a.x.hash(&mut h);
            coords_b.x.hash(&mut h);
        } else {
            coords_b.x.hash(&mut h);
            coords_a.x.hash(&mut h);
        }

        if coords_a.y > coords_b.y {
            coords_a.y.hash(&mut h);
            coords_b.y.hash(&mut h);
        } else {
            coords_b.y.hash(&mut h);
            coords_a.y.hash(&mut h);
        }

        h.finish()
    }

    fn get_sector_seed(&self, coords: &SectorCoord) -> u64 {
        let mut h = DefaultHasher::default();
        self.cfg.seed.hash(&mut h);
        coords.x.hash(&mut h);
        coords.y.hash(&mut h);
        h.finish()
    }

    // s = 3
    // -1, 0 = (-1, 0) (2, 0)
    // -2, 0 = (-1, 0) (1, 0)
    // -3, 0 = (-1, 0) (0, 0)
    // -4, 0 = (-2, 0) (2, 0)
    pub fn get_sector_coord(&self, coord: &GlobalCoord) -> (SectorCoord, LocalCoord) {
        fn compute(s: u32, v: i32) -> (i32, i32) {
            let s = s as i32;

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

        let (sectorx, cellx) = compute(self.cfg.sector_size, coord.x);
        let (sectory, celly) = compute(self.cfg.sector_size, coord.y);

        (V2I::new(sectorx, sectory), V2I::new(cellx, celly))
    }

    pub fn get_portals(&self, coords: &GlobalCoord) -> Vec<GlobalCoord> {
        commons::grid::get_4_neighbours(coords)
            .into_iter()
            .map(|(_, c)| c)
            .filter(|i| {
                self.portals.contains(&(coords.clone(), i.clone()))
                    || self.portals.contains(&(i.clone(), coords.clone()))
            })
            .collect()
    }
}

fn main() {
    let mut g = InfiniteGrid::new(Cfg::default());

    let room = g.get(&V2I::new(0, 0));
    print!("#");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sector_coord() {
        let size = 5i32;
        let mut g = InfiniteGrid::new(Cfg {
            seed: 0,
            portal_prob: 0.5,
            sector_size: size as u32,
        });
        let last_i = size - 1;

        // first sector
        assert_eq!(
            ([0, 0].into(), [0, 0].into()),
            g.get_sector_coord(&[0, 0].into())
        );
        assert_eq!(
            ([0, 0].into(), [1, 0].into()),
            g.get_sector_coord(&[1, 0].into())
        );
        assert_eq!(
            ([0, 0].into(), [3, 2].into()),
            g.get_sector_coord(&[3, 2].into())
        );
        // next sector
        assert_eq!(
            ([1, 0].into(), [0, last_i].into()),
            g.get_sector_coord(&[size, last_i].into())
        );
        // bottom down sector
        assert_eq!(
            ([1, 1].into(), [0, 0].into()),
            g.get_sector_coord(&[size, size].into())
        );
        // negative
        assert_eq!(
            ([-1, 0].into(), [last_i, 0].into()),
            g.get_sector_coord(&[-1, 0].into())
        );
        assert_eq!(
            ([0, -1].into(), [0, last_i].into()),
            g.get_sector_coord(&[0, -1].into())
        );
        assert_eq!(
            ([0, -1].into(), [0, last_i].into()),
            g.get_sector_coord(&[0, -1].into())
        );
        assert_eq!(
            ([-1, -1].into(), [last_i, last_i].into()),
            g.get_sector_coord(&[-1, -1].into())
        );
        assert_eq!(
            ([-1, 0].into(), [0, 0].into()),
            g.get_sector_coord(&[-size, 0].into())
        );
        assert_eq!(
            ([-2, 0].into(), [last_i, 0].into()),
            g.get_sector_coord(&[-size - 1, 0].into())
        );
    }

    #[test]
    fn test_empty() {
        let mut g = InfiniteGrid::new(Cfg::default());
        let coords = GlobalCoord::new(0, 0);

        let room = g.get(&coords);
        assert!(room.is_none());

        g.create(&coords);
        // assert_eq!(0, room.coords.x);
        // assert_eq!(0, room.coords.y);
        //
        let portals = g.get_portals(&coords);
        assert!(portals.len() > 0);
    }

    #[test]
    fn test_get_sectors_connection_seed() {
        let g = InfiniteGrid::new(Cfg::default());
        let a = [41, -53].into();
        let b = [41, -52].into();

        let h1 = g.get_sectors_connection_seed(&a, &b);
        let h2 = g.get_sectors_connection_seed(&b, &a);
        assert_eq!(h1, h2);
        assert_ne!(0, h1);

        let c = [40, -53].into();
        let h3 = g.get_sectors_connection_seed(&a, &c);
        assert_ne!(h1, h3);

        let d = [-41, 53].into();
        let e = [-41, 52].into();
        let h4 = g.get_sectors_connection_seed(&d, &e);
        assert_ne!(h1, h4);
    }

    #[test]
    fn test_get_sector_connections() {
        let g = InfiniteGrid::new(Cfg::default());

        let a = [41, -53].into();
        let b = [41, -52].into();

        let connections_a = g.get_border_connection_cells(&a, &b);
        let connections_b = g.get_border_connection_cells(&b, &a);

        assert_eq!(connections_a, connections_b);
        assert!(connections_a.len() > 0);
    }
}
