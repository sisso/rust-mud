use std::path::Path;

use socket_server::server_socket::SocketServer;
use socket_server::local_server::LocalServer;
use http_server::HttpServer;

use commons::DeltaTime;
use mud_engine::Engine;

use crate::command_line_controller::CommandLineController;
use crate::http_controller::HttpController;

#[derive(Debug)]
pub struct Params {
    pub data_dir: String,
    pub local: bool,
    pub socket: bool,
    pub http: bool,
}

// TODO: read from params what servers to start and forward configurations, like port
pub fn run(params: Params) {
    let mut engine: Engine = Engine::new();
    engine.load(params.data_dir.as_str());

    let mut local_controller = if params.local {
        let local_server = LocalServer::new();
        Some(CommandLineController::new(Box::new(local_server)))
    } else { None };

    let mut socket_controller = if params.socket {
        let socket_server = SocketServer::new();
        Some(CommandLineController::new(Box::new(socket_server)))
    } else { None };

    let mut http_controller = if params.http {
        let http_server = HttpServer::new();
        Some(HttpController::new(http_server))
    } else { None };

    let delta_time = DeltaTime(0.1);

    loop {
        if let Some(controller) = &mut local_controller {
            controller.handle_inputs(&mut engine);
        }
        if let Some(controller) = &mut socket_controller {
            controller.handle_inputs(&mut engine);
        }
        if let Some(controller) = &mut http_controller {
            controller.handle_inputs(&mut engine);
        }

        engine.tick(delta_time);

        let events = engine.take_events();
        if let Some(controller) = &mut local_controller {
            controller.handle_events(&mut engine, &events);
        }
        if let Some(controller) = &mut socket_controller {
            controller.handle_events(&mut engine, &events);
        }
        if let Some(controller) = &mut http_controller {
            controller.handle_events(&mut engine, &events);
        }
    }
}
