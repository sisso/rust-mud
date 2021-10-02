pub mod commons;
pub mod grid;
pub mod infinite_grid;
pub mod mixable_tuple;
pub mod random_grid;

use crate::commons::{RectI, V2I};
use crate::infinite_grid::GridPrinter;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use std::collections::HashSet;

struct State {
    grid_cfg: infinite_grid::Cfg,
    pos: V2I,
    view_distance: i32,
    grid: infinite_grid::InfiniteGrid,
    know_cells: HashSet<V2I>,
}

impl State {
    pub fn move_if_valid(&mut self, x: i32, y: i32) {
        let new_pos = self.pos.translate(x, y);
        if self.grid.has_portal(&self.pos, &new_pos) {
            self.pos = new_pos;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        match ctx.key {
            Some(VirtualKeyCode::Up) => {
                self.move_if_valid(0, -1);
            }
            Some(VirtualKeyCode::Right) => {
                self.move_if_valid(1, 0);
            }
            Some(VirtualKeyCode::Down) => {
                self.move_if_valid(0, 1);
            }
            Some(VirtualKeyCode::Left) => {
                self.move_if_valid(-1, 0);
            }
            _ => {}
        }

        let tl = self
            .pos
            .translate(-self.view_distance / 2, -self.view_distance / 2);
        let view = RectI::new(tl.x, tl.y, self.view_distance, self.view_distance);

        {
            let portals = self.grid.slice_grid(&view);
            let r = infinite_grid::TRoom {
                rect: view.clone(),
                portals,
            };

            self.know_cells.insert(self.pos.clone());

            let b = infinite_grid::print_rooms(&r);
            for (y, line) in b.split("\n").enumerate() {
                for (x, ch) in line.chars().enumerate() {
                    // let global = view.to_global(&(x as i32, y as i32).into());
                    //
                    // let rgb = if self.know_cells.contains(&global) {
                    //     RGB::from_f32(1.0, 0.5, 1.0)
                    // } else {
                    //     RGB::from_f32(1.0, 1.0, 1.0)
                    // };

                    let rgb = RGB::from_f32(1.0, 1.0, 1.0);
                    ctx.set(x, y, rgb, RGB::from_f32(0., 0., 0.), rltk::to_cp437(ch));
                }
            }

            let pos_at_view = view.to_local(&self.pos);

            // set character * 2 as print rooms print one cell each 2 rooms
            ctx.set(
                pos_at_view.x * 2 + 1,
                pos_at_view.y * 2 + 1,
                RGB::from_f32(0.0, 1.0, 1.0),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('@'),
            );
        }

        {
            let lp = [20, 20];
            let gp = GridPrinter::new(&view, &self.grid);

            for y in 0..gp.get_height() {
                for x in 0..gp.get_height() {
                    let px = gp.chars_per_cell() * x;
                    let py = gp.chars_per_cell() * y;

                    let b = gp.get_cell(x, y);
                    ctx.print(lp[0] + px, lp[1] + py, b[0..3].iter().collect::<String>());
                    ctx.print(
                        lp[0] + px,
                        lp[1] + py + 1,
                        b[3..6].iter().collect::<String>(),
                    );
                    ctx.print(
                        lp[0] + px,
                        lp[1] + py + 2,
                        b[6..9].iter().collect::<String>(),
                    );
                }
            }

            let pos_at_view = view.to_local(&self.pos);

            ctx.set(
                lp[0] + pos_at_view.x * 2 + 3,
                lp[1] + pos_at_view.y * 2 + 3,
                RGB::from_f32(0.0, 1.0, 1.0),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('@'),
            );
        }
    }
}

fn new_grid(cfg: &infinite_grid::Cfg) -> infinite_grid::InfiniteGrid {
    infinite_grid::InfiniteGrid::new(cfg.clone())
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let gcfg = infinite_grid::Cfg::default();
    let grid = new_grid(&gcfg);
    let gs = State {
        grid_cfg: gcfg,
        pos: (0, 0).into(),
        view_distance: 5,
        grid: grid,
        know_cells: Default::default(),
    };
    rltk::main_loop(context, gs)
}
