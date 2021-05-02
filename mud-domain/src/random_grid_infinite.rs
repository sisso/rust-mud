/*



use crate::game::domain::Dir;

pub struct Doors {
    pub n: bool,
    pub e: bool,
    pub w: bool,
    pub s: bool,
}

pub struct Cell {
    pub coords: [i32; 2],
    pub doors: Doors,
}

pub struct Cfg {
    pub portal_prob: f32,
    pub min_open: u32,
}

pub struct InfiniteRandomGrid {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    portals: Vec<(usize, usize)>,
}

impl InfiniteRandomGrid {
    pub fn new(cfg: Cfg) -> Self {
        InfiniteRandomGrid {
            width: 0,
            height: 0,
            cells: vec![],
            portals: vec![],
        }
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &'a Cell> + 'a {
        todo!()
    }

    pub fn get<'a>(&'a self, coords: [i32; 2]) -> Option<&'a Cell> {
        todo!()
    }

    pub fn neighbors(&self, coords: [i32; 2]) -> Vec<[i32; 2]> {
        todo!()
    }

    pub fn dimension() -> [i32; 2] {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_irg() -> InfiniteRandomGrid {
        InfiniteRandomGrid::new(Cfg {
            portal_prob: 0.5,
            min_open: 2,
        })
    }

    #[test]
    pub fn test_basic() {
        // let irg = new_irg();
        // let cells = irg.list().collect::<Vec<_>>();
        // assert_eq!(1, cells.len());
        // assert_eq!([0, 0], irg.get(cells[0].coords).unwrap().coords);
        // assert!(irq.get_doors(cells[0]).len() > 0);
        //
        // let c2 = irq.get_doors(cells[0])[0];
        // _ = irq.get_coord(c2);
        // assert_eq!(2, cells.len());
    }
}
*/
