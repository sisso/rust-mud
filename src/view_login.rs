use crate::player_connection::*;
use crate::player::*;

fn ask_name(connection: &mut PlayerConnection) -> std::io::Result<String> {
    loop {
        let provided_name = connection.read_field("login: ")?;

        if provided_name.len() <= 3 {
            let msg = format!("invalid login name '{}' Try again.", provided_name);
            connection.writeln(&msg)?;
            continue;
        }

        connection.writeln(format!("nice name {}", &provided_name).as_str())?;
        return Ok(provided_name);
    }
}

pub fn handle_login(id: u32, mut connection: PlayerConnection) -> std::io::Result<Player> {
    let name: String = ask_name(&mut connection)?;
    Ok(Player::new(id, name, connection))
}
