use crate::player_connection::*;

pub fn handle(mut player: PlayerConnection) -> std::io::Result<()> {
    loop {
        player.connection.write("\n> ");
        let line = player.connection.read_line()?;

        println!("{}/{} sends '{}'", player.id, player.login, line);

        if line.starts_with("quit") {
            return Ok(());
        } else {
            player.connection.writeln(format!("Unknown command '{}'", line).as_ref())?;
        }
    }
}
