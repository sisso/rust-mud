use super::commons::V2I;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

pub const DIR_ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];
pub type GridCoord = V2I;
pub type GridIndex = usize;

/**
    0 1 2
    3 4 5
    6 7 8
*/

#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub width: u32,
    pub height: u32,
    pub list: Vec<Option<T>>,
}

impl<T> Grid<T> {
    pub fn new_square(size: u32) -> Self {
        Grid::new(size, size)
    }

    pub fn new(width: u32, height: u32) -> Self {
        let mut list = vec![];
        for _ in 0..width * height {
            list.push(None);
        }

        Grid {
            width,
            height,
            list,
        }
    }

    pub fn set(&mut self, index: GridIndex, value: Option<T>) {
        assert!(self.is_valid_index(index));
        self.list[index as usize] = value;
    }

    pub fn set_at(&mut self, coord: &GridCoord, value: Option<T>) {
        assert!(self.is_valid_coords(coord));
        let index = self.coords_to_index(coord);
        self.list[index as usize] = value;
    }

    // TODO: should it exists?
    pub fn get(&self, index: u32) -> Option<&T> {
        self.list[index as usize].as_ref()
    }

    pub fn get_at(&self, coord: &GridCoord) -> Option<&T> {
        assert!(self.is_valid_coords(&coord));
        let index = self.coords_to_index(coord);
        self.list[index as usize].as_ref()
    }

    pub fn is_valid_index(&self, index: usize) -> bool {
        index < self.list.len()
    }

    pub fn is_valid_coords(&self, coords: &GridCoord) -> bool {
        coords.x >= 0
            && coords.y >= 0
            && coords.x < self.width as i32
            && coords.y < self.height as i32
    }

    // TODO: should return option?
    pub fn coords_to_index(&self, coords: &GridCoord) -> GridIndex {
        (coords.y * (self.width as i32) + coords.x) as usize
    }

    pub fn get_valid_4_neighbours(&self, coords: &GridCoord) -> Vec<GridCoord> {
        get_4_neighbours(coords)
            .into_iter()
            .map(|(_, i)| i)
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn get_valid_8_neighbours(&self, coords: &GridCoord) -> Vec<GridCoord> {
        get_8_neighbours(coords)
            .into_iter()
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn raytrace(&self, pos: GridCoord, dir_x: i32, dir_y: i32) -> Vec<GridCoord> {
        let mut current = pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = (nx, ny).into();

            if !self.is_valid_coords(&current) {
                break;
            }

            match self.get_at(&current) {
                Some(_) => result.push(current),
                None => break,
            }
        }

        result
    }
}

/// return sequentially with DIR_ALL
pub fn get_4_neighbours(coords: &GridCoord) -> Vec<(Dir, GridCoord)> {
    vec![
        (Dir::N, coords.translate(0, -1)),
        (Dir::E, coords.translate(1, 0)),
        (Dir::S, coords.translate(0, 1)),
        (Dir::W, coords.translate(-1, 0)),
    ]
}

pub fn get_8_neighbours(coords: &GridCoord) -> Vec<GridCoord> {
    vec![
        coords.translate(0, -1),
        coords.translate(1, -1),
        coords.translate(1, 0),
        coords.translate(1, 1),
        coords.translate(0, 1),
        coords.translate(-1, 1),
        coords.translate(-1, 0),
        coords.translate(-1, -1),
    ]
}

#[derive(Debug, Clone)]
pub struct FlexGrid<T> {
    pub cells: HashMap<GridCoord, T>,
}

impl<T> FlexGrid<T> {
    pub fn new() -> Self {
        FlexGrid {
            cells: HashMap::new(),
        }
    }

    pub fn set_at(&mut self, coord: &GridCoord, value: Option<T>) {
        match value {
            Some(v) => self.cells.insert(coord.to_owned(), v),
            None => self.cells.remove(coord),
        };
    }

    pub fn get_at(&self, coord: &GridCoord) -> Option<&T> {
        self.cells.get(coord)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_grid_get_neighbors() {
        let neighbours = get_4_neighbours(&GridCoord::new(0, 0));
        assert_eq!(
            neighbours,
            vec![
                (Dir::N, GridCoord::new(0, -1)),
                (Dir::E, GridCoord::new(1, 0)),
                (Dir::S, GridCoord::new(0, 1)),
                (Dir::W, GridCoord::new(-1, 0)),
            ]
        );
    }

    #[test]
    pub fn test_grid_get_valid_neighbors() {
        let grid = Grid::<u32>::new(2, 2);
        let neighbours = grid.get_valid_8_neighbours(&GridCoord::new(0, 0));
        assert_eq!(
            vec![
                GridCoord::new(1, 0),
                GridCoord::new(1, 1),
                GridCoord::new(0, 1),
            ],
            neighbours
        );
    }

    #[test]
    pub fn test_grid_raytrace() {
        let mut grid = Grid::<u32>::new(4, 2);

        // X###
        // ###
        assert_eq!(grid.raytrace((0, 0).into(), -1, 0), Vec::<GridCoord>::new());

        // #X##
        // ####
        assert_eq!(grid.raytrace((1, 0).into(), -1, 0), Vec::<GridCoord>::new());

        // 0###
        // ####
        grid.set_at(&(0, 0).into(), Some(0));

        // 0X##
        // ####
        assert_eq!(grid.raytrace((1, 0).into(), -1, 0), vec![(0, 0).into()]);

        // 00##
        // ####
        grid.set_at(&(1, 0).into(), Some(0));

        // 00X#
        // ####
        assert_eq!(
            grid.raytrace((2, 0).into(), -1, 0),
            vec![(1, 0).into(), (0, 0).into()]
        );

        // 00#X
        // ####
        assert_eq!(grid.raytrace((3, 0).into(), -1, 0), vec![]);

        // X0##
        // ####
        assert_eq!(grid.raytrace((0, 0).into(), 1, 0), vec![(1, 0).into()]);
    }
}
