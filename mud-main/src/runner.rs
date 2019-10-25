use std::path::Path;
use socket_server::server_socket::SocketServer;
use http_server::HttpServer;
use crate::command_line_controller::CommandLineController;
use crate::http_controller::HttpController;
use socket_server::local_server::LocalServer;
use core::utils::DeltaTime;

// TODO: move to proper package
pub struct Engine {

}

impl Engine {
    pub fn new() -> Self {
        Engine {}
    }

    pub fn load(&mut self, data_dir: &str) {

    }

    pub fn tick(&mut self, delta_time: DeltaTime) {

    }
}

#[derive(Debug)]
pub struct Params {
    pub data_dir: String
}

pub fn run(params: Params) {
    let mut engine = Engine::new();
    engine.load(params.data_dir.as_str());

    let mut local_server = LocalServer::new();
    let mut socket_server = SocketServer::new();
    let mut http_server = HttpServer::new();

    let mut command_line_controller = CommandLineController::new(vec![
        Box::new(local_server),
        Box::new(socket_server)
    ]);
    let mut http_controller = HttpController::new(http_server);

    let delta_time = DeltaTime(0.1);

    loop {
        command_line_controller.handle(&mut engine);
        http_controller.handle(&mut engine);
        engine.tick(delta_time);
    }
}
