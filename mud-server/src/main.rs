extern crate commons;
extern crate mud_domain;
extern crate rand;
extern crate socket_server;

pub mod game_server;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let module = arguments
        .get(1)
        .map(|i| i.as_str())
        .unwrap_or("./data/space");

    crate::game_server::run(module);
}
