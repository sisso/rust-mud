use super::{
    commons::{self, RectI, V2I},
    grid, random_grid,
};
use crate::mixable_tuple::MixableTuple;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

type GlobalCoord = commons::V2I;
type SectorCoord = commons::V2I;
type LocalCoord = commons::V2I;

#[derive(Debug)]
pub struct Sector {}

impl Sector {}

#[derive(Debug, Clone)]
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

type Portal = MixableTuple<GlobalCoord>;

#[derive(Debug)]
pub struct InfiniteGrid {
    cfg: Cfg,
    sectors: HashMap<SectorCoord, Sector>,
    portals: HashSet<Portal>,
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

    pub fn create(&mut self, coord: &GlobalCoord) {
        let (sector_coord, _) = self.global_to_local(coord);

        match self.sectors.get_mut(&sector_coord) {
            None => {
                self.new_sector(&sector_coord);
            }
            _ => {}
        }
    }

    pub fn create_all(&mut self, rect: &RectI) {
        for x in rect.get_top_left().x..rect.get_bottom_right().x {
            for y in rect.get_top_left().y..rect.get_bottom_right().y {
                self.create(&(x, y).into());
            }
        }
    }

    fn new_sector(&mut self, sector_coords: &SectorCoord) {
        // create sector
        let cfg = random_grid::RandomGridCfg {
            width: self.cfg.sector_size as usize,
            height: self.cfg.sector_size as usize,
            portal_prob: self.cfg.portal_prob,
            deep_levels: 0,
        };

        let mut rng: StdRng = SeedableRng::seed_from_u64(self.get_sector_seed(&sector_coords));
        let rgrid = random_grid::LevelGrid::new(&cfg, &mut rng);
        let topleft = self.local_to_global(sector_coords, &(0, 0).into());

        let portals = rgrid.get_portals().iter().map(|(a, b)| {
            fn to_v2i((x, y): (usize, usize)) -> V2I {
                V2I::new(x as i32, y as i32)
            }

            let ca = to_v2i(rgrid.get_coords(*a));
            let cb = to_v2i(rgrid.get_coords(*b));

            (
                ca.translate(topleft.x, topleft.y),
                cb.translate(topleft.x, topleft.y),
            )
        });

        for (a, b) in portals {
            self.portals.insert(MixableTuple::new(a, b));
        }

        // create portals to other sectors
        for (a, b) in self.get_sector_connection_cells(sector_coords) {
            self.portals.insert(MixableTuple::new(a, b));
        }

        self.sectors.insert(sector_coords.clone(), Sector {});
    }

    fn get_sector_connection_cells(
        &self,
        sector_cords: &SectorCoord,
    ) -> Vec<(GlobalCoord, GlobalCoord)> {
        grid::get_4_neighbours(&sector_cords)
            .into_iter()
            .flat_map(|(dir, sector)| {
                let cells = self.get_border_connection_cells(sector_cords, &sector);

                // collect local position of borders into a specific direaction
                let ((x, y), local_points): ((i32, i32), Vec<LocalCoord>) = match dir {
                    grid::Dir::N => (
                        (0, -1),
                        cells.into_iter().map(|index| V2I::new(index, 0)).collect(),
                    ),
                    grid::Dir::E => (
                        (1, 0),
                        cells
                            .into_iter()
                            .map(|index| V2I::new(self.cfg.sector_size as i32 - 1, index))
                            .collect(),
                    ),
                    grid::Dir::S => (
                        (0, 1),
                        cells
                            .into_iter()
                            .map(|index| V2I::new(index, self.cfg.sector_size as i32 - 1))
                            .collect(),
                    ),
                    grid::Dir::W => (
                        (-1, 0),
                        cells.into_iter().map(|index| V2I::new(0, index)).collect(),
                    ),
                };

                local_points
                    .into_iter()
                    .map(|local| {
                        let g1 = self.local_to_global(sector_cords, &local);
                        let g2 = g1.translate(x, y);
                        (g1, g2)
                    })
                    .collect::<Vec<_>>()
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

    pub fn global_to_local(&self, coord: &GlobalCoord) -> (SectorCoord, LocalCoord) {
        // s = 3
        // -1, 0 = (-1, 0) (2, 0)
        // -2, 0 = (-1, 0) (1, 0)
        // -3, 0 = (-1, 0) (0, 0)
        // -4, 0 = (-2, 0) (2, 0)
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
                let mut cell = v % s;
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

    /// return global coordinate giving a sector and local
    pub fn local_to_global(
        &self,
        sector_coords: &SectorCoord,
        local_coord: &LocalCoord,
    ) -> GlobalCoord {
        let top_left = V2I::new(
            sector_coords.x * self.cfg.sector_size as i32,
            sector_coords.y * self.cfg.sector_size as i32,
        );

        top_left.translate(local_coord.x, local_coord.y)
    }

    pub fn has_portal(&self, coords_a: &GlobalCoord, coords_b: &GlobalCoord) -> bool {
        let mt = MixableTuple::new(coords_a.to_owned(), coords_b.to_owned());
        self.portals.contains(&mt)
    }

    pub fn get_portals(&self, coords: &GlobalCoord) -> Vec<GlobalCoord> {
        grid::get_4_neighbours(coords)
            .into_iter()
            .map(|(_, c)| c)
            .filter(|i| self.has_portal(coords, i))
            .collect()
    }

    pub fn slice_grid(&mut self, rect: &RectI) -> HashSet<MixableTuple<GlobalCoord>> {
        for x in rect.get_top_left().x..rect.get_bottom_right().x {
            for y in rect.get_top_left().y..rect.get_bottom_right().y {
                self.create(&(x, y).into());
            }
        }

        let portals = self
            .portals
            .iter()
            .filter(|mt| {
                let f = mt.get_a();
                let t = mt.get_b();
                rect.is_inside(f) || rect.is_inside(t)
            })
            .cloned()
            .collect();

        portals
    }
}

pub struct GridPrinter<'a> {
    pub rect: &'a RectI,
    pub grid: &'a InfiniteGrid,
}

impl<'a> GridPrinter<'a> {
    pub fn new(rect: &'a RectI, grid: &'a InfiniteGrid) -> Self {
        GridPrinter { rect, grid }
    }

    pub fn get_width(&self) -> i32 {
        self.rect.get_width()
    }
    pub fn get_height(&self) -> i32 {
        self.rect.get_height()
    }

    pub fn chars_per_cell(&self) -> i32 {
        3
    }
    pub fn get_cell(&self, x: i32, y: i32) -> [char; 9] {
        let empty = '.';
        let portal_v = '|';
        let portal_h = '=';
        let room = '#';
        let mut buffer: [char; 9] = [
            empty, empty, empty, empty, empty, empty, empty, empty, empty,
        ];

        let global = self.rect.to_global(&(x, y).into());

        let portal_n = self.is_portal(global.x, global.y, global.x, global.y - 1);
        let portal_e = self.is_portal(global.x + 1, global.y, global.x, global.y);
        let portal_w = self.is_portal(global.x - 1, global.y, global.x, global.y);
        let portal_s = self.is_portal(global.x, global.y, global.x, global.y + 1);

        if portal_n {
            buffer[1] = portal_v;
        }

        if portal_e {
            buffer[3 + 2] = portal_h;
        }

        if portal_s {
            buffer[6 + 1] = portal_v;
        }

        if portal_w {
            buffer[3] = portal_h;
        }

        if portal_n || portal_e || portal_s || portal_w {
            buffer[4] = room;
        }

        buffer
    }

    fn is_portal(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
        let pa = (x0, y0).into();
        let pb = (x1, y1).into();
        self.grid.has_portal(&pa, &pb)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_global_to_local() {
        let g = InfiniteGrid::new(Cfg {
            seed: 0,
            portal_prob: 0.0,
            sector_size: 3,
        });

        let use_cases = get_local_global_cases();
        for (expected_sector, expected_local, global) in use_cases {
            let (sector, local) = g.global_to_local(&global.into());

            assert_eq!(
                V2I::from(expected_sector),
                sector,
                "invalid sector coordinate for {:?}",
                global
            );

            assert_eq!(
                V2I::from(expected_local),
                local,
                "invalid local coordinate for {:?}",
                global
            );
        }
    }

    /// (sector_coord, local_coord, global_coord)
    fn get_local_global_cases() -> Vec<((i32, i32), (i32, i32), (i32, i32))> {
        let use_cases = vec![
            ((0, 0), (0, 0), (0, 0)),
            ((0, 0), (1, 0), (1, 0)),
            ((0, 0), (2, 0), (2, 0)),
            ((1, 0), (0, 0), (3, 0)),
            ((1, 0), (1, 0), (4, 0)),
            ((1, 0), (2, 0), (5, 0)),
            ((0, 0), (0, 1), (0, 1)),
            ((0, 0), (0, 2), (0, 2)),
            ((0, 1), (0, 0), (0, 3)),
            ((-1, 0), (2, 0), (-1, 0)),
            ((-1, 0), (1, 0), (-2, 0)),
            ((-1, 0), (0, 0), (-3, 0)),
            ((-2, 0), (2, 0), (-4, 0)),
            ((0, -1), (0, 2), (0, -1)),
            ((0, -1), (0, 1), (0, -2)),
            ((0, -1), (0, 0), (0, -3)),
            ((0, -2), (0, 2), (0, -4)),
        ];
        use_cases
    }

    #[test]
    fn test_local_to_global() {
        let g = InfiniteGrid::new(Cfg {
            seed: 0,
            portal_prob: 0.0,
            sector_size: 3,
        });

        let use_cases = get_local_global_cases();

        for (sector, local, expected_global) in use_cases {
            assert_eq!(
                V2I::from(expected_global),
                g.local_to_global(&sector.into(), &local.into()),
            )
        }
    }

    #[test]
    fn test_empty() {
        let mut g = InfiniteGrid::new(Cfg::default());
        let coords = GlobalCoord::new(0, 0);

        g.create(&coords);
        println!("{:?}", g);

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

    #[test]
    fn test_slice_grid() {
        let mut g = InfiniteGrid::new(Cfg {
            seed: 0,
            portal_prob: 0.5,
            sector_size: 3,
        });

        let tl = GlobalCoord::new(-2, -2);
        let br = GlobalCoord::new(4, 5);
        let rect = RectI::new_2_points(tl, br);
        let portals = g.slice_grid(&rect);
        assert!(portals.len() > 0)
    }

    #[test]
    fn test_has_portal() {
        let mut g = InfiniteGrid::new(Cfg {
            seed: 0,
            portal_prob: 0.5,
            sector_size: 3,
        });

        let tl = GlobalCoord::new(-2, -2);
        let br = GlobalCoord::new(4, 5);
        let rect = RectI::new_2_points(tl, br);
        let portals = g.slice_grid(&rect);
        let buff = print_rooms(&TRoom {
            rect: rect.clone(),
            portals,
        });

        let g = grid::Grid::<()>::new(rect.get_width() as u32, rect.get_height() as u32);
        let coord = g.coords_to_index(&[rect.get_width() / 2, rect.get_height() / 2].into());

        let buff: String = buff
            .chars()
            .enumerate()
            .map(|(i, c)| if i == coord { '@' } else { c })
            .collect();

        println!("{}", buff)
    }
}
