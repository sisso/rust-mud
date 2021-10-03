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
    pos: V2I,
    grid: infinite_grid::InfiniteGrid,
    know_cells: HashSet<V2I>,
    show_global: bool,
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
            Some(VirtualKeyCode::Space) => {
                self.show_global = !self.show_global;
            }
            _ => {}
        }

        self.know_cells.insert(self.pos.clone());

        let show_local = true;
        let all_pos = [0, 0];
        let all_view_distance = 17;
        let local_view_distance = 5;
        let local_pos = [
            all_view_distance + local_view_distance / 2 - 1,
            all_view_distance + local_view_distance / 2 - 1,
        ];

        if self.show_global {
            let fg_char = RGB::from_f32(0.0, 0.0, 1.0);
            let fg = RGB::from_f32(0.5, 0.5, 0.5);
            let bg = RGB::from_f32(0., 0., 0.);

            let tl = self
                .pos
                .translate(-all_view_distance / 2, -all_view_distance / 2);
            let view = RectI::new(tl.x, tl.y, all_view_distance, all_view_distance);

            let gp = GridPrinter::new(&view, &self.grid);
            for y in 0..gp.get_height() {
                for x in 0..gp.get_height() {
                    let px = gp.chars_per_cell() * x;
                    let py = gp.chars_per_cell() * y;

                    let b = gp.get_cell(x, y);
                    ctx.print_color(
                        all_pos[0] + px,
                        all_pos[1] + py,
                        fg,
                        bg,
                        b[0..3].iter().collect::<String>(),
                    );
                    ctx.print_color(
                        all_pos[0] + px,
                        all_pos[1] + py + 1,
                        fg,
                        bg,
                        b[3..6].iter().collect::<String>(),
                    );
                    ctx.print_color(
                        all_pos[0] + px,
                        all_pos[1] + py + 2,
                        fg,
                        bg,
                        b[6..9].iter().collect::<String>(),
                    );

                    let gp = view.to_global(&(x, y).into());
                    if self.know_cells.contains(&gp) {
                        ctx.set(
                            all_pos[0] + px + 1,
                            all_pos[1] + py + 1,
                            fg_char,
                            bg,
                            rltk::to_cp437('#'),
                        );
                    }
                }
            }
        }

        if show_local {
            let fg = RGB::from_f32(1.0, 1.0, 1.0);
            let bg = RGB::from_f32(0., 0., 0.);

            let tl = self
                .pos
                .translate(-local_view_distance / 2, -local_view_distance / 2);
            let view = RectI::new(tl.x, tl.y, local_view_distance, local_view_distance);

            self.grid.create_all(&view);
            let gp = GridPrinter::new(&view, &self.grid);

            for y in 0..gp.get_height() {
                for x in 0..gp.get_height() {
                    let px = gp.chars_per_cell() * x;
                    let py = gp.chars_per_cell() * y;

                    let b = gp.get_cell(x, y);
                    ctx.print_color(
                        local_pos[0] + px,
                        local_pos[1] + py,
                        fg,
                        bg,
                        b[0..3].iter().collect::<String>(),
                    );
                    ctx.print_color(
                        local_pos[0] + px,
                        local_pos[1] + py + 1,
                        fg,
                        bg,
                        b[3..6].iter().collect::<String>(),
                    );
                    ctx.print_color(
                        local_pos[0] + px,
                        local_pos[1] + py + 2,
                        fg,
                        bg,
                        b[6..9].iter().collect::<String>(),
                    );

                    let gp = view.to_global(&(x, y).into());
                    if self.know_cells.contains(&gp) {
                        ctx.set(
                            local_pos[0] + px + 1,
                            local_pos[1] + py + 1,
                            RGB::from_f32(0.0, 0.0, 1.0),
                            RGB::from_f32(0., 0., 0.),
                            rltk::to_cp437('#'),
                        );
                    }
                }
            }

            let pos_at_view = view.to_local(&self.pos);

            ctx.set(
                local_pos[0] + pos_at_view.x * 2 + 3,
                local_pos[1] + pos_at_view.y * 2 + 3,
                RGB::from_f32(0.0, 1.0, 1.0),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('@'),
            );
        }

        ctx.print(0, 0, "press SPACE to see all cells");
    }
}

fn new_grid(cfg: &infinite_grid::Cfg) -> infinite_grid::InfiniteGrid {
    infinite_grid::InfiniteGrid::new(cfg.clone())
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Infintie grid")
        .build()?;
    let gcfg = infinite_grid::Cfg::default();
    let grid = new_grid(&gcfg);
    let gs = State {
        pos: (0, 0).into(),
        grid: grid,
        know_cells: Default::default(),
        show_global: false,
    };
    rltk::main_loop(context, gs)
}
