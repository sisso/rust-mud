use socket_server::ConnectionId;
use mud_engine::Engine;
use core::utils::PlayerId;

use super::comm;
use crate::command_line_controller::ControllerAction;

pub enum ViewKind {
    Login,
    CharacterCreation,
    Menu,
    Game
}

pub enum ViewAction {
    None,
    ChangeView,
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
    pub player_id: Option<PlayerId>,
    pub current: ViewKind,
}

///
/// State of all views
///
impl ViewData {
    pub fn new(connection_id: ConnectionId) -> Self {
        ViewData {
            connection_id,
            player_id: None,
            current: ViewKind::Login,
        }
    }
}

///
/// Contains all data related to a single connection
///
pub struct ViewContext {
    pub data: ViewData,
    pub view_login: LoginView,
    pub view_menu: MenuView,
    pub view_character_creation: CharacterCreationView,
}

impl ViewContext {
    pub fn new(connection_id: ConnectionId) -> Self {
        ViewContext {
            data: ViewData::new(connection_id),
            view_login: LoginView::new(),
            view_menu: MenuView::new(),
            view_character_creation: CharacterCreationView::new(),
        }
    }

    pub fn init(&mut self, view_manager: &mut dyn ViewController) {
        match self.data.current {
            ViewKind::Login => self.view_login.init(view_manager, &mut self.data),
            ViewKind::Menu => self.view_menu.init(view_manager, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.init(view_manager, &mut self.data),
            _ => panic!(),
        }
    }

    pub fn handle(&mut self, view_manager: &mut dyn ViewController, mut input: String) {
        let input = input.trim();

        let action = match self.data.current {
            ViewKind::Login => self.view_login.handle(view_manager, input, &mut self.data),
            ViewKind::Menu => self.view_menu.handle(view_manager, input, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.handle(view_manager, input, &mut self.data),
            _ => panic!(),
        };

        match action {
            ViewAction::ChangeView => self.init(view_manager),
            ViewAction::None => {},
        }
    }
}

///
/// Provide access to the rest of engine to a View
///
/// Can be partitioned into per view Controller
///
pub trait ViewController {
    fn output(&mut self, connection_id: ConnectionId, msg: String);
    fn execute_login(&mut self, connection_id: ConnectionId, login: &str, pass: &str) -> Result<PlayerId, ()>;
    fn disconnect(&mut self, connection_id: ConnectionId);
}

///
/// Full responsible to update ViewData, including setting player id
///
pub trait View {
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData);

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction;
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
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::welcome());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        match self.login.take() {
            Some(login) => {
                match view_manager.execute_login(data.connection_id, login.as_str(), input) {
                    Ok(player_id) => {
                        data.player_id = Some(player_id);
                        data.current = ViewKind::Menu;
                        ViewAction::ChangeView
                    },
                    Err(_) => {
                        view_manager.output(data.connection_id, comm::login_fail(login.as_str()));
                        view_manager.output(data.connection_id, comm::login_request_login());
                        ViewAction::None
                    }
                }
            },
            None if input.eq("new") => {
                data.current = ViewKind::CharacterCreation;
                ViewAction::ChangeView
            },
            None => {
                if input.len() < 3 {
                    view_manager.output(data.connection_id, comm::login_invalid(input));
                    view_manager.output(data.connection_id, comm::login_request_login());
                } else {
                    self.login = Some(input.to_string());
                    view_manager.output(data.connection_id, comm::login_request_password());
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
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::menu_welcome());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        match input {
            "1" => {
                data.current = ViewKind::Game;
                ViewAction::ChangeView
            },
            "2" => {
                view_manager.disconnect(data.connection_id);
                ViewAction::None
            },
            other => {
                view_manager.output(data.connection_id, comm::menu_invalid(input));
                ViewAction::None
            },
        }
    }
}

pub struct CharacterCreationView {

}

impl CharacterCreationView {
    pub fn new() -> Self {
        CharacterCreationView {}
    }
}

impl View for CharacterCreationView {
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        // TODO: query game classes
        // TODO: query game races
        // TODO: create character for player

        ViewAction::None
    }
}
