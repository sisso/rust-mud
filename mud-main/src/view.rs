use socket_server::ConnectionId;
use mud_engine::Engine;
use core::utils::UserId;

use crate::command_line_controller::{Outputs, ViewData};
use crate::comm;

enum ViewKind {
    Login,
    CharacterCreation,
    Menu,
    Game
}

enum ViewAction {
    None,
    SetView { kind: ViewKind },
    Disconnect,
}

pub trait ViewManager {
    fn output(&mut self, connection_id: ConnectionId, msg: String);
    fn execute_login(&mut self, login: &str, pass: &str) -> Result<UserId, ()>;
    fn disconnect(&mut self, connection_id: ConnectionId);
}

pub trait View {
    fn init(view_manager: &mut dyn ViewManager, data: &mut ViewData);

    fn handle(view_manager: &mut dyn ViewManager, data: &mut ViewData, input: String) -> ViewAction;
}

pub struct LoginView {

}

impl View for LoginView {
    fn init(view_manager: &mut dyn ViewManager, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::welcome());
    }

    fn handle(view_manager: &mut dyn ViewManager, data: &mut ViewData, input: String) -> ViewAction {
        match data.login_data.login.take() {
            Some(login) => {
                match view_manager.execute_login(login.as_str(), input.as_str()) {
                    Ok(user_id) => {
                        data.user_id = Some(user_id);
                        ViewAction::SetView { kind: ViewKind::Menu }
                    },
                    Err(_) => {
                        view_manager.output(data.connection_id, comm::login_fail(login.as_str()));
                        ViewAction::None
                    }
                }
            },
            None => {
                if input.len() < 3 {
                    view_manager.output(data.connection_id, comm::login_invalid(input.as_str()));
                } else {
                    data.login_data.login = Some(input);
                }

                ViewAction::None
            }
        }
    }
}

pub struct MenuView {

}

impl View for MenuView {
    fn init(view_manager: &mut dyn ViewManager, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::menu_welcome());
    }

    fn handle(view_manager: &mut dyn ViewManager, data: &mut ViewData, input: String) -> ViewAction {
        match input.as_str() {
            "1" => {
                ViewAction::SetView { kind: ViewKind::Game }
            },
            "2" => {
                ViewAction::Disconnect
            },
            other => {
                ViewAction::None
            },
        }
    }
}