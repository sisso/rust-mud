use std::collections::HashSet;
use std::collections::HashMap;

pub struct Game {
    next_mob_id: u32,
    rooms: Vec<Room>,
    mobs: Vec<Mob>,
    players: Vec<GamePlayer>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            next_mob_id: 0,
            rooms: vec![],
            mobs: vec![],
            players: vec![],
        }
    }
}

pub struct GamePlayer {
    pub id: u32,
    pub login: String,
    pub avatar_id: u32
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u32,
    pub room_id: u32,
    pub label: String,
    pub tags: HashSet<MobTag>,
}

#[derive(Clone, Debug)]
pub enum Dir {
    N,
    S,
    W,
    E
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum MobTag {
    AVATAR
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum RoomTag {
    INITIAL
}

#[derive(Clone, Debug)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub exits: Vec<(Dir, u32)>,
    pub tags: HashSet<RoomTag>,
}

impl Game {
    pub fn player_connect(&mut self, id: u32, login: String, avatar_id: u32) {
        println!("adding player {}/{}", id, login);

        self.players.push(GamePlayer {
            id,
            login,
            avatar_id,
        });
    }

    pub fn player_disconnect(&mut self, id: u32) {
        println!("removing player {}", id);

        let index = self.players.iter().position(|x| x.id == id).unwrap();
        self.players.remove(index);
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

    pub fn get_rooms_by_tag(&self, tag: &RoomTag) -> Vec<u32> {
        self.rooms
            .iter()
            .filter(|room| room.tags.contains(tag))
            .map(|room| room.id)
            .collect()
    }

    pub fn get_room(&self, id: u32) -> Room {
        let room = self.rooms.iter().find(|room| { room.id == id }).unwrap();
        room.clone()
    }

    pub fn new_mob(&mut self, room_id: u32, label: String) -> Mob {
        Mob {
            id: self.next_mob_id(),
            label: label,
            room_id: room_id,
            tags: HashSet::new()
        }
    }

    pub fn add_mob(&mut self, mob: Mob) {
        self.mobs.push(mob);
    }
}


impl Game {
    fn next_mob_id(&mut self) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id += 1;
        id
    }
}
