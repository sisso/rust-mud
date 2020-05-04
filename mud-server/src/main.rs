extern crate commons;
extern crate mud_domain;
extern crate rand;
extern crate socket_server;

use std::process::exit;

pub mod game_server;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();

    if arguments.len() < 2 {
        println!("{} <module> [<profile>]", arguments.get(0).unwrap());
        println!();
        println!("./data/fantasy");
        exit(1);
    }

    let module = arguments.get(1).unwrap();
    let profile = arguments.get(2).cloned();

    crate::game_server::start_server(module, profile);
}
