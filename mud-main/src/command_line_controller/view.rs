use socket_server::ConnectionId;
use mud_engine::Engine;
use core::utils::UserId;

use super::Outputs;
use super::comm;

pub enum ViewKind {
    Login,
    CharacterCreation,
    Menu,
    Game
}

pub enum ViewAction {
    None,
    SetView { kind: ViewKind },
    Disconnect,
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
}

impl ViewData {
    pub fn new(connection_id: ConnectionId) -> Self {
        ViewData {
            connection_id,
            user_id: None,
            current: ViewKind::Login,
        }
    }
}

pub trait ViewManager<'a> {
    fn output(&mut self, connection_id: ConnectionId, msg: String);
    fn execute_login(&mut self, login: &str, pass: &str) -> Result<UserId, ()>;
}

pub trait View {
    fn init(&mut self, view_manager: &mut dyn ViewManager, data: &mut ViewData);

    fn handle(&mut self, view_manager: &mut dyn ViewManager, input: String, data: &mut ViewData) -> ViewAction;
}

pub struct LoginView {
    login: Option<String>
}

impl LoginView {
    pub fn new() -> Self {
        LoginView { login: None }
    }
}

impl View for LoginView {
    fn init(&mut self, view_manager: &mut dyn ViewManager, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::welcome());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewManager, input: String, data: &mut ViewData) -> ViewAction {
        match self.login.take() {
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
                    self.login = Some(input);
                }

                ViewAction::None
            }
        }
    }
}

pub struct MenuView {

}

impl MenuView {
    pub fn new() -> Self {
        MenuView {}
    }
}

impl View for MenuView {
    fn init(&mut self, view_manager: &mut dyn ViewManager, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::menu_welcome());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewManager, input: String, data: &mut ViewData) -> ViewAction {
        match input.as_str() {
            "1" => {
                ViewAction::SetView { kind: ViewKind::Game }
            },
            "2" => {
                ViewAction::Disconnect
            },
            other => {
                view_manager.output(data.connection_id, comm::menu_invalid(input.as_str()));
                ViewAction::None
            },
        }
    }
}
