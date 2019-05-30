use crate::player_connection::PlayerConnection;

pub struct Player {
    pub id: u32,
    pub name: String,
    pub connection: PlayerConnection,
}

impl Player {
    pub fn new(id: u32, name: String, connection: PlayerConnection) -> Self {
        Player {
            id,
            name,
            connection,
        }
    }
}
