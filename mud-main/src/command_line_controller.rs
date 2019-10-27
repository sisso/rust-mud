mod view;
mod comm;

use std::collections::HashMap;
use std::borrow::BorrowMut;

use socket_server::{Server, ServerOutput, ServerChanges};
use commons::{PlayerId, vec_take, ConnectionId};
use mud_engine::{Engine, ConnectionEvent};

use view::*;

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
            let view = self.connections.get_mut(&connection_id).unwrap();
            view.init(&mut view_controller);
        }

        for connection_id in result.disconnects {
            let view = self.connections.remove(&connection_id).unwrap();
            for player_id in view.data.player_id.into_iter(){
                self.connection_per_player_id.remove(&player_id);
            }
        }

        for input in result.inputs {
            let view = self.connections.get_mut(&input.connection_id).unwrap();
            view.handle(&mut view_controller, input.msg);
        }

        self.flush(view_controller.take());
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<ConnectionEvent>) {
        let mut view_controller = CommandLineViewController::new(engine);
//        handle_outputs(engine, &mut self.outputs, events);
        self.flush(view_controller.take());
    }

    // TODO: normalize new lines?
    // TODO: show prompt?
    // TODO: Be able to pre-append new lines before any output (where player has send no message)
    //       but is receiving a output
    fn flush(&mut self, actions: Vec<ControllerAction>) {

        for action in actions {
            match action {
                ControllerAction::Output { connection_id, msg } => {
                    self.server.output(connection_id, msg);
                },
                ControllerAction::Login { connection_id, player_id } => {
                    self.connection_per_player_id.insert(player_id, connection_id);
                },
                ControllerAction::Logout { connection_id } => {
                    let view = self.connections.get(&connection_id).unwrap();
                    if let Some(player_id) = &view.data.player_id {
                        self.connection_per_player_id.remove(&player_id);
                    }
                    self.connections.remove(&connection_id);
                    self.server.disconnect(connection_id);
                },

            }
        }
    }
}

enum ControllerAction {
    Login  { connection_id: ConnectionId, player_id: PlayerId },
    Logout { connection_id: ConnectionId },
    Output { connection_id: ConnectionId, msg: String },
}

/// Temporary instance used to collect actions that need to be applied
/// to CommandLineController and Engine
/// 
/// Give access to engine data
struct CommandLineViewController<'a> {
    engine: &'a mut Engine,
    actions: Vec<ControllerAction>,
}

impl<'a> CommandLineViewController<'a> {
    fn new(engine: &'a mut Engine) -> Self {
        CommandLineViewController {
            engine,
            actions: Vec::new(),
        }
    }

    fn take(&mut self) -> Vec<ControllerAction> {
        std::mem::replace(&mut self.actions, Vec::new())
    }
}

impl<'a> ViewController for CommandLineViewController<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.actions.push(ControllerAction::Output {
            connection_id,
            msg
        });
    }

    fn execute_login(&mut self, connection_id: ConnectionId, login: &str, pass: &str) -> Result<PlayerId, ()> {
        self.engine.login(login, pass).map(|player_id| {
            self.actions.push(ControllerAction::Login {
                connection_id,
                player_id
            });
            player_id
        })
    }

    fn disconnect(&mut self, connection_id: ConnectionId) {
        self.actions.push(ControllerAction::Logout {
            connection_id,
        });
    }
}
