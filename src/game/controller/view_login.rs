
pub struct HandleResult {
    pub login: Option<String>,
    pub msg: String,
}

pub fn handle(input: String) -> HandleResult {
    if input.len() > 3 {
        let msg= format!("login success, welcome {}\n\n", input);

        HandleResult {
           login: Some(input),
           msg,
        }
    } else {
        HandleResult {
            login :None,
            msg: format!("invalid login {}\n\nlogin: ", input)
        }
    }
}

pub fn handle_welcome() -> String {
    "Welcome to MUD\n--------------\n\nlogin: ".to_string()
}
