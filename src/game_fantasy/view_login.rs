
pub fn handle(input: String) -> (Option<String>, String) {
    if input.len() > 3 {
        let msg= format!("login success, welcome {}\n\n", input);
        (Some(input), msg)
    } else {
        (None, format!("invalid login {}\n\nlogin: ", input))
    }
}

pub fn handle_welcome() -> String {
    "Welcome to MUD\n--------------\n\nlogin: ".to_string()
}
