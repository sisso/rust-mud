use crate::player_connection::PlayerConnection;
use crate::player::Player;

pub struct Game {
    rooms: Vec<Room>,
    players: Vec<GamePlayer>
}

pub struct GamePlayer {
    pub id: u32,
    pub login: String
}

pub enum Dir {
    N,
    S,
    W,
    E
}

pub struct Room {
    pub id: u32,
    pub name: String,
    pub exits: Vec<(Dir, u32)>
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: vec![],
            rooms: vec![]
        }
    }

    pub fn player_connect(&mut self, id: u32, login: String) {
        println!("adding to game {}/{}", id, login);

        self.players.push(GamePlayer {
            id,
            login
        });
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }
}
