pub fn welcome() -> String {
    r#"--== Welcome to Mud ==--

To create a new characer, type new in login.

login: "#.to_string()
}

pub fn login_request_login() -> String {
    format!("login: ")
}

pub fn login_request_password() -> String {
    format!("password: ")
}

pub fn login_fail(login: &str) -> String {
    format!("Fail to login '{}', verify if your login and password are correctly.\n", login)
}

pub fn login_invalid(login: &str) -> String {
    format!("Invalid login '{}', it must be at least 4 characters.\n", login)
}

pub fn menu_welcome() -> String {
    r#"-= Menu =-
  1) Enter
  2) Disconnect
"#.to_string()
}

pub fn menu_invalid(input: &str) -> String {
    format!("Invalid option '{}'.\n", input)
}

pub fn prompt() -> String {
    format!("$ ")
}

pub fn unknown_input(input: &str) -> String {
    format!("unknown input '{}'", input)
}
