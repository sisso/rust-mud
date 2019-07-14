extern crate rand;

mod server;
mod socket_server;
mod game;
mod lib;

fn main() {
    crate::game::run();
}
