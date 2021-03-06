pub enum LoginResult {
    Msg { msg: String },
    Login { login: String },
}

pub fn handle(input: &str) -> LoginResult {
    if input.len() > 3 {
        LoginResult::Login {
            login: input.to_string(),
        }
    } else {
        LoginResult::Msg {
            msg: format!("invalid login {}\n\nlogin: ", input),
        }
    }
}

pub fn on_login_success(login: &str) -> String {
    format!("login success, welcome back {}\n\n", login)
}

pub fn handle_welcome() -> String {
    "Welcome to MUD\n--------------\n\nlogin: ".to_string()
}
