pub fn welcome() -> String {
    r#"--== Welcome to Mud ==--

To create a new characer, type new in login.

login: "#.to_string()
}

pub fn login_fail(login: &str) -> String {
    format!("Fail to login '{}', verify if your login and password are correctly.", login)
}

pub fn login_invalid(login: &str) -> String {
    format!("Invalid login '{}', it must be at least 4 characters.", login)
}

pub fn menu_welcome() -> String {
    r#"-= Menu =-
  1) Enter
  2) Disconnect
"#.to_string()
}

pub fn menu_invalid(input: &str) -> String {
    format!("Invalid option '{}'.", input)
}
