use crate::runner::Params;

mod runner;
mod command_line_controller;
mod http_controller;

fn main() {
    runner::run(Params {
        data_dir: "./data/mud-fantasy".to_string()
    });
}
