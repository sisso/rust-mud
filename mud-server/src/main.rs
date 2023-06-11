extern crate commons;
extern crate mud_domain;
extern crate rand;
extern crate socket_server;

use log::LevelFilter;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::server_runner::ServerConfig;

pub mod http_handler;
pub mod server_runner;

fn main() {
    env_logger::builder().filter(None, LevelFilter::Info).init();

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

    let mut server = crate::server_runner::create_server(
        ServerConfig {
            socket_port: 3333,
            http_port: 8333,
            data_folder: std::path::PathBuf::from("data-live"),
            module_path: std::path::PathBuf::from(module),
            profile: profile.cloned(),
        },
        stop_flag,
    )
    .expect("fail to open server");

    server.run().expect("fail to run the server");
}
