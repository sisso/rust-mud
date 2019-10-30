use super::*;

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
