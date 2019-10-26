use socket_server::{Server, ConnectionId, ServerOutput, ServerChanges};
use core::utils::{UserId, vec_take};
use mud_engine::{Engine, Output};
use std::collections::HashMap;
use std::borrow::BorrowMut;
use crate::command_line_controller::view::{ViewData, LoginView, MenuView, View, ViewManager, ViewKind, ViewAction};

// TODO: how to normalize outputs to add $hp $

//mod input_handler;
//mod output_handler;
mod view;
mod comm;

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

struct ConnectionView {
    data: ViewData,
    view_login: LoginView,
    view_menu: MenuView,
}

impl ConnectionView {
    pub fn new(connection_id: ConnectionId) -> Self {
        ConnectionView {
            data: ViewData::new(connection_id),
            view_login: LoginView::new(),
            view_menu: MenuView::new(),
        }
    }
}

pub struct CommandLineController {
    server: Box<dyn Server>,
    connections: HashMap<ConnectionId, ConnectionView>,
}

impl CommandLineController {
    pub fn new(server: Box<dyn Server>) -> Self {
        CommandLineController {
            server,
            connections: Default::default(),
        }
    }

//    fn login() {
//        let user_id = engine.add_connection();
//        let _ = self.per_user_id.insert(user_id, connection_id);
//        let _ = self.per_connection_id.insert(connection_id, user_id);
//        let mut view = ViewData::new(connection_id);
//        init_login(engine, &mut self.outputs, &mut view);
//        let _ = self.view_data.insert(user_id, view);
//    }
//
//    fn disconnect() {
//        let user_id = self.per_connection_id.remove(&connection_id).unwrap();
//        let _ = self.per_user_id.remove(&user_id);
//        engine.remove_connection(user_id);
//    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {
        let mut view_manager = ControllerViewManager::new(engine);

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
        let mut view_manager = ControllerViewManager::new(engine);
//        handle_outputs(engine, &mut self.outputs, events);
        view_manager.flush(&mut *self.server);
    }
}

struct ControllerViewManager<'a> {
    engine: &'a mut Engine,
    buffer: Vec<(ConnectionId, String)>,
}

impl<'a> ControllerViewManager<'a> {
    pub fn new(engine: &'a mut Engine) -> Self {
        ControllerViewManager {
            engine,
            buffer: Vec::new(),
        }
    }

    pub fn flush(&mut self, server: &mut Server) {
        let buffer = std::mem::replace(&mut self.buffer, Vec::new());
        for (connection_id, msg) in buffer {
            server.output(connection_id, msg);
        }
    }
}

impl<'a> ViewManager<'a> for ControllerViewManager<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.buffer.push((connection_id, msg));
    }

    fn execute_login(&mut self, login: &str, pass: &str) -> Result<UserId, ()> {
        let user_id = self.engine.add_connection();
        Ok(user_id)
    }
}
