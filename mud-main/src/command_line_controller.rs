use socket_server::{Server, ConnectionId};
use core::utils::{UserId, vec_take};
use core_engine::{Engine, Output};
use std::collections::HashMap;
use crate::command_line_controller::input_handler::handle_input;
use crate::command_line_controller::output_handler::handle_outputs;

mod input_handler;
mod output_handler;

pub struct CommandLineController {
    server: Box<dyn Server>,
    per_user_id: HashMap<UserId, ConnectionId>,
    per_connection_id: HashMap<ConnectionId, UserId>,
}

impl CommandLineController {
    pub fn new(server: Box<dyn Server>) -> Self {
        CommandLineController {
            server,
            per_user_id: Default::default(),
            per_connection_id: Default::default(),
        }
    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {
        let result = self.server.run();

        for connection_id in result.connects {
            let user_id = engine.add_connection();
            self.per_user_id.insert(user_id, connection_id);
            self.per_connection_id.insert(connection_id, user_id);
        }

        for connection_id in result.disconnects {
            let user_id = self.per_connection_id.remove(&connection_id).unwrap();
            let _ = self.per_user_id.remove(&user_id);
            engine.remove_connection(user_id);
        }

        for input in result.inputs {
            let user_id = self.per_connection_id.get(&input.connection_id).unwrap();
            handle_input(engine, *user_id, input.msg);
        }
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<Output>) {
        let messages = handle_outputs(engine, &self.per_user_id, events);
        self.server.append_output(messages);
    }
}
