use socket_server::{Server, ConnectionId, ServerOutput, ServerChanges};
use core::utils::{PlayerId, vec_take};
use mud_engine::{Engine, Output};
use std::collections::HashMap;
use std::borrow::BorrowMut;
use crate::command_line_controller::view::{ViewData, LoginView, MenuView, View, ViewController, ViewKind, ViewAction, CharacterCreationView, ViewContext};

// TODO: how to normalize outputs to add $hp $

mod view;
mod comm;

///
/// When calling View, it split some of its fields into ControllerViewManager to allow
/// map View actions into mutability.
///
pub struct CommandLineController {
    server: Box<dyn Server>,
    connections: HashMap<ConnectionId, ViewContext>,
    connection_per_player_id: HashMap<PlayerId, ConnectionId>,
}

impl CommandLineController {
    pub fn new(server: Box<dyn Server>) -> Self {
        CommandLineController {
            server,
            connections: Default::default(),
            connection_per_player_id: Default::default(),
        }
    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {
        let mut view_controller = CommandLineViewController::new(engine);

        let result = self.server.run();

        for connection_id in result.connects {
            self.connections.insert(connection_id, ViewContext::new(connection_id));
            let mut view = self.connections.get_mut(&connection_id).unwrap();
            view.init(&mut view_controller);
        }

        for connection_id in result.disconnects {
            let view = self.connections.remove(&connection_id).unwrap();
            for player_id in view.data.player_id.into_iter(){
                self.connection_per_player_id.remove(&player_id);
            }
        }

        for input in result.inputs {
            let mut view = self.connections.get_mut(&input.connection_id).unwrap();
            view.handle(&mut view_controller, input.msg);
        }

        for (player_id, connection_id) in view_controller.take_login() {
            self.connection_per_player_id.insert(player_id, connection_id);
        }

        view_controller.flush(&mut *self.server);
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<Output>) {
        let mut view_manager = CommandLineViewController::new(engine);
//        handle_outputs(engine, &mut self.outputs, events);
        view_manager.flush(&mut *self.server);
    }
}

/// Used to map view commands into CommandLineController and Engine
///
/// Is create temporary when needed
///
struct CommandLineViewController<'a> {
    engine: &'a mut Engine,
    buffer: Vec<(ConnectionId, String)>,
    disconnects_request: Vec<ConnectionId>,
    logins: Vec<(PlayerId, ConnectionId)>,
}

impl<'a> CommandLineViewController<'a> {
    pub fn new(engine: &'a mut Engine) -> Self {
        CommandLineViewController {
            engine,
            buffer: Vec::new(),
            disconnects_request: Vec::new(),
            logins: Vec::new(),
        }
    }

    pub fn flush(&mut self, server: &mut dyn Server) {
        let buffer = std::mem::replace(&mut self.buffer, Vec::new());
        for (connection_id, msg) in buffer {
            server.output(connection_id, msg);
        }

        for connection_id in std::mem::replace(&mut self.disconnects_request, Vec::new()) {
            server.disconnect(connection_id);
        }
    }

    pub fn take_login(&mut self) -> Vec<(PlayerId, ConnectionId)> {
        std::mem::replace(&mut self.logins, Vec::new())
    }
}

impl<'a> ViewController for CommandLineViewController<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.buffer.push((connection_id, msg));
    }

    fn execute_login(&mut self, connection_id: ConnectionId, login: &str, pass: &str) -> Result<PlayerId, ()> {
        self.engine.login(login, pass).map(|player_id| {
            self.logins.push((player_id, connection_id));
            player_id
        })
    }

    fn disconnect(&mut self, connection_id: ConnectionId) {
        self.disconnects_request.push(connection_id);
    }
}
