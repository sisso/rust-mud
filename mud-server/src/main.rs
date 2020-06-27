extern crate commons;
extern crate mud_domain;
extern crate rand;
extern crate socket_server;

use crate::game_server::ServerConfig;
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

    let module = &arguments[1];
    let profile = arguments.get(2);

    crate::game_server::start_server(ServerConfig {
        port: 3333,
        data_folder: std::path::PathBuf::from(module),
        module_path: std::path::PathBuf::from("/tmp/mud"),
        profile: profile.cloned(),
    })
    .expect("fail to open server");
}
