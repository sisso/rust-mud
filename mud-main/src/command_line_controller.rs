mod view;
mod comm;

use std::collections::{HashMap, HashSet};
use std::borrow::BorrowMut;

use socket_server::{Server, ServerOutput, ServerChanges};
use commons::{PlayerId, ConnectionId};
use mud_engine::{Engine, ConnectionEvent, Action};
use logs::*;
use view::*;

///
/// When calling View, it split some of its fields into ControllerViewManager to allow
/// map View actions into mutability.
///
pub struct CommandLineController {
    server: Box<dyn Server>,
    connections: HashMap<ConnectionId, ViewContext>,
    connection_per_player_id: HashMap<PlayerId, ConnectionId>,
    connections_with_input: HashSet<ConnectionId>,
}

impl CommandLineController {
    pub fn new(server: Box<dyn Server>) -> Self {
        CommandLineController {
            server,
            connections: Default::default(),
            connection_per_player_id: Default::default(),
            connections_with_input: Default::default(),
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
            for player_id in view.data.player_id.into_iter() {
                self.connection_per_player_id.remove(&player_id);
            }
        }

        // clean up before update with new values
        self.connections_with_input.clear();

        for input in result.inputs {
            self.connections_with_input.insert(input.connection_id);

            let view = self.connections.get_mut(&input.connection_id).unwrap();
            view.handle(&mut view_controller, input.msg);
        }


        self.flush(view_controller.take());
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<ConnectionEvent>) {
        let mut view_controller = CommandLineViewController::new(engine);

        for event in events {
            let player_id = event.player_id;
            match self.connection_per_player_id.get(&player_id) {
                Some(connection_id) => {
                   let connection = self.connections.get_mut(connection_id).unwrap();
                   connection.handle_events(&mut view_controller, &event.events);
                },
                None => {
                    // player could belong to another connector
                },
            }
        }
        self.flush(view_controller.take());
    }

    // TODO: normalize new lines?
    // TODO: show prompt?
    // TODO: Be able to pre-append new lines before any output (where player has send no message)
    //       but is receiving a output
    // TODO: how disable text normalization for request entries?
    // TOOD: how mix event and request entries?
    fn flush(&mut self, actions: Vec<ControllerAction>) {
        let mut outputs: Vec<(ConnectionId, String)> = Vec::new();

        for action in actions {
            match action {
                ControllerAction::Output { connection_id, msg } => {
                    outputs.push((connection_id, msg));
                },
                ControllerAction::Login { connection_id, player_id } => {
                    info!("{:?} login {:?}", connection_id, player_id);
                    self.connection_per_player_id.insert(player_id, connection_id);
                },
                ControllerAction::Logout { connection_id } => {
                    let view = self.connections.get(&connection_id).unwrap();
                    if let Some(player_id) = &view.data.player_id {
                        info!("{:?} logout {:?}", connection_id, player_id);
                        self.connection_per_player_id.remove(&player_id);
                    } else {
                        info!("{:?} logout", connection_id);
                    }
                    self.connections.remove(&connection_id);
                    self.server.disconnect(connection_id);
                },
            }
        }

        let outputs = normalize_outputs(&self.connections_with_input, outputs);
        for (connection_id, msg) in outputs {
            self.server.output(connection_id, msg);
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
    controller_actions: Vec<ControllerAction>,
}

impl<'a> CommandLineViewController<'a> {
    fn new(engine: &'a mut Engine) -> Self {
        CommandLineViewController {
            engine,
            controller_actions: Vec::new(),
        }
    }

    fn take(&mut self) -> Vec<ControllerAction> {
        std::mem::replace(&mut self.controller_actions, Vec::new())
    }
}

impl<'a> view::login_view::LoginController for CommandLineViewController<'a> {
    fn output(&mut self, msg: String) {
        unimplemented!()
    }

    fn login(&mut self, login: &str, pass: &str) -> bool {
        unimplemented!()
    }

    fn set_view_create_character() {
        unimplemented!()
    }
}

impl<'a> ViewController for CommandLineViewController<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.controller_actions.push(ControllerAction::Output {
            connection_id,
            msg
        });
    }

    fn execute_login(&mut self, connection_id: ConnectionId, login: &str, pass: &str) -> Result<PlayerId, ()> {
        self.engine.login(login, pass).map(|player_id| {
            self.controller_actions.push(ControllerAction::Login {
                connection_id,
                player_id
            });
            player_id
        })
    }

    fn disconnect(&mut self, connection_id: ConnectionId) {
        self.controller_actions.push(ControllerAction::Logout {
            connection_id,
        });
    }

    fn emit(&mut self, player_id: PlayerId, action: Action) {
        self.engine.add_action(player_id, action);
    }
}

fn normalize_outputs(has_inputs: &HashSet<ConnectionId>, mut outputs: Vec<(ConnectionId, String)>) -> Vec<(ConnectionId, String)> {
    let mut result = Vec::new();
    let mut connections_with_output = HashSet::new();

    // prepend messages
    for (connection_id, _) in outputs.iter() {
        connections_with_output.insert(*connection_id);

        if !has_inputs.contains(&connection_id) {
            result.push((*connection_id, "".to_string()));
        }
    }

    // add other messages
    result.append(&mut outputs);

    // add new lines and prompts
    for connection_id in connections_with_output {
        result.push((connection_id, "$ ".to_string()));
    }

    result
}

#[cfg(test)]
mod test {
//    use super::*;

//    #[test]
//    fn test_normalize_output_should_add_prompt_in_end() {
//        unimplemented!()
//    }
//
//    #[test]
//    fn test_normalize_output_should_pre_append_new_line_if_connection_dont_send_input() {
//        unimplemented!()
//    }
}
