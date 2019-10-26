use crate::runner::Params;

mod runner;
mod command_line_controller;
mod http_controller;
mod view;
mod comm;

fn main() {
    runner::run(Params {
        data_dir: "./data/mud-fantasy".to_string()
    });
}
