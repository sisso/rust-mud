extern crate rand;

mod server;
mod server_dummy;
mod server_socket;
mod game;
mod lib;

fn main() {
    crate::game::run();
}
