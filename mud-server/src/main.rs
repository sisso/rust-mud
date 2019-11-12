extern crate rand;
extern crate commons;
extern crate mud_domain;
extern crate socket_server;

pub mod game_server;

fn main() {
    crate::game_server::run();
}

