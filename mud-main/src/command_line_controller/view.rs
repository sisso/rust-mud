mod character_creation_view;
mod game_view;
mod login_view;
mod menu_view;

use mud_engine::Engine;
use commons::{PlayerId, ConnectionId};

use super::comm;
use crate::command_line_controller::ControllerAction;
use crate::command_line_controller::view::login_view::LoginView;
use crate::command_line_controller::view::menu_view::MenuView;
use crate::command_line_controller::view::character_creation_view::CharacterCreationView;
use crate::command_line_controller::view::game_view::GameView;

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
    pub view_game: GameView,
}

impl ViewContext {
    pub fn new(connection_id: ConnectionId) -> Self {
        ViewContext {
            data: ViewData::new(connection_id),
            view_login: LoginView::new(),
            view_menu: MenuView::new(),
            view_character_creation: CharacterCreationView::new(),
            view_game: GameView::new(),
        }
    }

    pub fn init(&mut self, view_manager: &mut dyn ViewController) {
        match self.data.current {
            ViewKind::Login => self.view_login.init(view_manager, &mut self.data),
            ViewKind::Menu => self.view_menu.init(view_manager, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.init(view_manager, &mut self.data),
            ViewKind::Game => self.view_game.init(view_manager, &mut self.data),
            _ => panic!(),
        }
    }

    pub fn handle(&mut self, view_manager: &mut dyn ViewController, input: String) {
        let input = input.trim();

        let action = match self.data.current {
            ViewKind::Login => self.view_login.handle(view_manager, input, &mut self.data),
            ViewKind::Menu => self.view_menu.handle(view_manager, input, &mut self.data),
            ViewKind::CharacterCreation => self.view_character_creation.handle(view_manager, input, &mut self.data),
            ViewKind::Game => self.view_game.handle(view_manager, input, &mut self.data),
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



