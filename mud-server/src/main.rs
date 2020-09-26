extern crate commons;
extern crate logs;
extern crate mud_domain;
extern crate rand;
extern crate socket_server;

use crate::game_server::ServerConfig;
use logs::*;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time::Duration};

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

    let stop_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    {
        let stop_flag = stop_flag.clone();
        ctrlc::set_handler(move || {
            stop_flag.store(true, Ordering::Relaxed);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let mut server = crate::game_server::create_server(
        ServerConfig {
            port: 3333,
            data_folder: std::path::PathBuf::from("data-live"),
            module_path: std::path::PathBuf::from(module),
            profile: profile.cloned(),
        },
        stop_flag,
    )
    .expect("fail to open server");

    server.run().expect("fail to run the server");
}
