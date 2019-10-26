use mud_engine::Engine;
use crate::command_line_controller::{Outputs, ViewData};
use socket_server::ConnectionId;
use crate::comm;
use core::utils::UserId;

enum ViewKind {
    Login,
    CharacterCreation,
    Menu,
    Game
}

pub trait ViewManager {
    fn output(&mut self, connection_id: ConnectionId, msg: String);
    fn push_view(&mut self, connection_id: ConnectionId, kind: ViewKind);
    fn pop_view(&mut self, connection_id: ConnectionId);
}

pub trait ConnectionView {
    fn output(&mut self, msg: String);
    fn push_view(&mut self, kind: ViewKind);
    fn set_view(&mut self, kind: ViewKind);
    fn pop_view(&mut self);
    fn get_view_data(&mut self) -> &mut ViewData;
    fn execute_login(&mut self, login: &str, pass: &str) -> Result<UserId, ()>;
    fn disconnect(&mut self);
}

pub trait View {
    fn init(view: &mut dyn ConnectionView);

    fn handle(view: &mut dyn ConnectionView, input: String);
}

pub struct LoginView {

}

impl View for LoginView {
    fn init(view: &mut dyn ConnectionView) {
        view.output(comm::welcome());
    }

    fn handle(view: &mut dyn ConnectionView, input: String) {
        let mut login= view.get_view_data().login_data.login.take();
        if let Some(login) = login {
            let login_result = view.execute_login(login.as_str(), input.as_str());
            match login_result {
                Ok(user_id) => {
                    let mut data = view.get_view_data();
                    data.user_id = Some(user_id);
                    view.set_view(ViewKind::Menu);
                },
                Err(_) => {
                    view.output(comm::login_fail(login.as_str()));
                }
            }
        } else {
            if input.len() < 3 {
                view.output(comm::login_invalid(input.as_str()));
            } else {
                let mut data = view.get_view_data();
                data.login_data.login = Some(input);
            }
        }
    }
}

pub struct MenuView {

}

impl View for MenuView {
    fn init(view: &mut dyn ConnectionView) {
        view.output(comm::menu_welcome());
    }

    fn handle(view: &mut dyn ConnectionView, input: String) {
        match input.as_str() {
            "1" => {
                view.set_view(ViewKind::Game);
            },
            "2" => {
                view.disconnect();
            },
            other => {},
        }
    }
}