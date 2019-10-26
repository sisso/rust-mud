use std::path::Path;

use socket_server::server_socket::SocketServer;
use socket_server::local_server::LocalServer;
use http_server::HttpServer;

use core::utils::DeltaTime;
use mud_engine::Engine;

use crate::command_line_controller::CommandLineController;
use crate::http_controller::HttpController;

#[derive(Debug)]
pub struct Params {
    pub data_dir: String
}

// TODO: read from params what servers to start and forward configurations, like port
pub fn run(params: Params) {
    let mut engine: Engine = Engine::new();
    engine.load(params.data_dir.as_str());

    let mut local_server = LocalServer::new();
    let mut socket_server = SocketServer::new();
    let mut http_server = HttpServer::new();

    let mut local_controller = CommandLineController::new(Box::new(local_server));
    let mut socket_controller = CommandLineController::new(Box::new(socket_server));
    let mut http_controller = HttpController::new(http_server);

    let delta_time = DeltaTime(0.1);

    loop {
        local_controller.handle_inputs(&mut engine);
        socket_controller.handle_inputs(&mut engine);
        http_controller.handle_inputs(&mut engine);

        engine.tick(delta_time);

        let events = engine.take_events();
        local_controller.handle_events(&mut engine, &events);
        socket_controller.handle_events(&mut engine, &events);
        http_controller.handle_events(&mut engine, &events);
    }
}
