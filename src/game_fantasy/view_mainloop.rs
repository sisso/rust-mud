/// return login
pub fn handle(login: &String, input: String) -> Option<String> {
    let output = format!("unknown command '{}'\n$ ", input);
    Some(output)
}
