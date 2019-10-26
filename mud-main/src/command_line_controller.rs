use socket_server::{Server, ConnectionId, ServerOutput, ServerChanges};
use core::utils::{PlayerId, vec_take};
use mud_engine::{Engine, Output};
use std::collections::HashMap;
use std::borrow::BorrowMut;
use crate::command_line_controller::view::{ViewData, LoginView, MenuView, View, ViewManager, ViewKind, ViewAction, CharacterCreationView};

// TODO: how to normalize outputs to add $hp $

mod view;
mod comm;

struct ConnectionView {
    data: ViewData,
    view_login: LoginView,
    view_menu: MenuView,
    view_character_creation: CharacterCreationView,
}

impl ConnectionView {
    pub fn new(connection_id: ConnectionId) -> Self {
        ConnectionView {
            data: ViewData::new(connection_id),
            view_login: LoginView::new(),
            view_menu: MenuView::new(),
            view_character_creation: CharacterCreationView::new(),
        }
    }

    pub fn init(&mut self, view_manager: &mut dyn ViewManager) {
        match self.data.current {
            ViewKind::Login => self.view_login.init(view_manager, &mut self.data),
            ViewKind::Menu => self.view_menu.init(view_manager, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.init(view_manager, &mut self.data),
            _ => panic!(),
        }
    }

    pub fn handle(&mut self, view_manager: &mut dyn ViewManager, input: String) -> ViewAction {
        match self.data.current {
            ViewKind::Login => self.view_login.handle(view_manager, input, &mut self.data),
            ViewKind::Menu => self.view_menu.handle(view_manager, input, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.handle(view_manager, input, &mut self.data),
            _ => panic!(),
        }
    }
}

pub struct CommandLineController {
    server: Box<dyn Server>,
    connections: HashMap<ConnectionId, ConnectionView>,
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

//    fn login() {
//        let player_id = engine.add_connection();
//        let _ = self.per_player_id.insert(player_id, connection_id);
//        let _ = self.per_connection_id.insert(connection_id, player_id);
//        let mut view = ViewData::new(connection_id);
//        init_login(engine, &mut self.outputs, &mut view);
//        let _ = self.view_data.insert(player_id, view);
//    }
//
//    fn disconnect() {
//        let player_id = self.per_connection_id.remove(&connection_id).unwrap();
//        let _ = self.per_player_id.remove(&player_id);
//        engine.remove_connection(player_id);
//    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {
        let mut view_manager = ControllerViewManager::new(engine, &mut self.connection_per_player_id);

        let result = self.server.run();

        for connection_id in result.connects {
            self.connections.insert(connection_id, ConnectionView::new(connection_id));
            let mut view = self.connections.get_mut(&connection_id).unwrap();
            view.view_login.init(&mut view_manager, &mut view.data);
        }

        for connection_id in result.disconnects {
            self.connections.remove(&connection_id);
        }

        for input in result.inputs {
            let mut view = self.connections.get_mut(&input.connection_id).unwrap();
            let action = match view.data.current {
                ViewKind::Login => {
                    view.view_login.handle(&mut view_manager, input.msg, &mut view.data)
                },
                ViewKind::Menu => {
                    view.view_menu.handle(&mut view_manager, input.msg, &mut view.data)
                },
                _ => {
                    ViewAction::Disconnect
                },
            };

            match action {
                ViewAction::None => {},
                ViewAction::Disconnect => {
                    self.connections.remove(&input.connection_id);
                    self.server.disconnect(input.connection_id);
                },
                ViewAction::SetView { kind } => {
                    view.data.current = kind;

                    let action = match view.data.current {
                        ViewKind::Login => view.view_login.init(&mut view_manager, &mut view.data),
                        ViewKind::Menu => view.view_menu.init(&mut view_manager, &mut view.data),
                        _ => panic!(),
                    };
                },
            }
        }

        view_manager.flush(&mut *self.server);
    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<Output>) {
        let mut view_manager = ControllerViewManager::new(engine, &mut self.connection_per_player_id);
//        handle_outputs(engine, &mut self.outputs, events);
        view_manager.flush(&mut *self.server);
    }
}

struct ControllerViewManager<'a> {
    engine: &'a mut Engine,
    buffer: Vec<(ConnectionId, String)>,
    connection_per_player_id: &'a mut HashMap<PlayerId, ConnectionId>,
}

impl<'a> ControllerViewManager<'a> {
    pub fn new(engine: &'a mut Engine, connection_per_player_id: &'a mut HashMap<PlayerId, ConnectionId>) -> Self {
        ControllerViewManager {
            engine,
            buffer: Vec::new(),
            connection_per_player_id,
        }
    }

    pub fn flush(&mut self, server: &mut dyn Server) {
        let buffer = std::mem::replace(&mut self.buffer, Vec::new());
        for (connection_id, msg) in buffer {
            server.output(connection_id, msg);
        }
    }
}

impl<'a> ViewManager for ControllerViewManager<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.buffer.push((connection_id, msg));
    }

    fn execute_login(&mut self, connection_id: ConnectionId, login: &str, pass: &str) -> Result<PlayerId, ()> {
        self.engine.login(login, pass).map(|player_id| {
            self.connection_per_player_id.insert(player_id, connection_id);
            player_id
        })
    }
}
