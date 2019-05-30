use crate::player_connection::PlayerConnection;

pub struct Player {
    pub name: String,
    pub connection: PlayerConnection,
}

impl Player {
    pub fn new(name: String, connection: PlayerConnection) -> Self {
        Player {
            name,
            connection,
        }
    }
}
