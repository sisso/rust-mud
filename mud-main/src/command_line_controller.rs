use socket_server::{Server, ConnectionId, ServerOutput};
use core::utils::{UserId, vec_take};
use mud_engine::{Engine, Output};
use std::collections::HashMap;
use crate::command_line_controller::input_handler::{handle_input, init_login};
use crate::command_line_controller::output_handler::handle_outputs;
use std::borrow::BorrowMut;

mod input_handler;
mod output_handler;

pub enum ViewKind {
    Login,
    Menu,
    CharacterCreation,
    Game,
}

pub struct LoginViewData {
    pub login: Option<String>,
}

impl LoginViewData {
    pub fn new() -> Self {
        LoginViewData { login: None }
    }
}

pub struct CharacterCreationViewData {
    race: Option<String>,
    class: Option<String>,
}

impl CharacterCreationViewData {
    pub fn new() -> Self {
        CharacterCreationViewData { race: None, class: None }
    }
}

pub struct GameViewData {

}

impl GameViewData {
    pub fn new() -> Self {
        GameViewData {}
    }
}

pub struct ViewData {
    pub connection_id: ConnectionId,
    pub user_id: Option<UserId>,
    pub current: ViewKind,
    pub login_data: LoginViewData,
    pub character_creation_data: CharacterCreationViewData,
    pub game_data: GameViewData,
}

impl ViewData {
    pub fn new(connection_id: ConnectionId) -> Self {
        ViewData {
            connection_id,
            user_id: None,
            current: ViewKind::Login,
            login_data: LoginViewData::new(),
            character_creation_data: CharacterCreationViewData::new(),
            game_data: GameViewData::new(),
        }
    }
}

pub trait Outputs {
    fn add(&mut self, connection_id: ConnectionId, msg: String);
}

pub struct OutputsBuffer {
    buffer: Vec<(ConnectionId, String)>,
}

impl OutputsBuffer {
    pub fn new() -> Self {
        OutputsBuffer { buffer: vec![] }
    }

    pub fn flush(&mut self, server: &mut Server) {
        let buffer = std::mem::replace(&mut self.buffer, Vec::new());
        for (connection_id, msg) in buffer {
            server.output(connection_id, msg);
        }
    }
}

impl Outputs for OutputsBuffer {
    fn add(&mut self, connection_id: ConnectionId, msg: String) {
        self.buffer.push((connection_id, msg));
    }
}

pub struct CommandLineController {
    server: Box<dyn Server>,
    per_user_id: HashMap<UserId, ConnectionId>,
    per_connection_id: HashMap<ConnectionId, UserId>,
    view_data: HashMap<UserId, ViewData>,
    outputs: OutputsBuffer,
}

impl CommandLineController {
    pub fn new(server: Box<dyn Server>) -> Self {
        CommandLineController {
            server,
            per_user_id: Default::default(),
            per_connection_id: Default::default(),
            view_data: Default::default(),
            outputs: OutputsBuffer::new(),
        }
    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {
        let result = self.server.run();

        for connection_id in result.connects {
            let user_id = engine.add_connection();
            let _ = self.per_user_id.insert(user_id, connection_id);
            let _ = self.per_connection_id.insert(connection_id, user_id);
            let mut view = ViewData::new(connection_id);
            init_login(engine, &mut self.outputs, &mut view);
            let _ = self.view_data.insert(user_id, view);
        }

        for connection_id in result.disconnects {
            let user_id = self.per_connection_id.remove(&connection_id).unwrap();
            let _ = self.per_user_id.remove(&user_id);
            engine.remove_connection(user_id);
        }

        for input in result.inputs {
            let user_id = self.per_connection_id.get(&input.connection_id).unwrap();
            let mut view = self.view_data.get_mut(&user_id).unwrap();
            handle_input(engine, &mut self.outputs, &mut view, input.msg);
        }

        self.outputs.flush(&mut *self.server);
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<Output>) {
        handle_outputs(engine, &mut self.outputs, events);
    }
}
