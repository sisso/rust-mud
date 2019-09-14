extern crate rand;

pub mod utils;
pub mod server;
pub mod server_dummy;
pub mod server_socket;
pub mod game;

mod main_server;

fn main() {
    crate::main_server::run();
}
