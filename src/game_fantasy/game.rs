use std::collections::HashSet;

pub struct Game {
    next_mob_id: u32,
    rooms: Vec<Room>,
    mobs: Vec<Mob>,
    players: Vec<Player>,
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

#[derive(Clone, Debug)]
pub struct Player {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dir {
    N,
    S,
    W,
    E
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Dir::N => write!(f, "N"),
            Dir::S => write!(f, "S"),
            Dir::E => write!(f, "E"),
            Dir::W => write!(f, "W"),
        }
    }
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
    pub label: String,
    pub desc: String,
    pub exits: Vec<(Dir, u32)>,
    pub tags: HashSet<RoomTag>,
}

impl Game {
    pub fn player_connect(&mut self, id: u32, login: &String, avatar_id: u32) {
        println!("adding player {}/{}", id, login);

        self.players.push(Player {
            id,
            login: login.clone(),
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
        let room = self.rooms
            .iter()
            .find(|room| { room.id == id })
            .unwrap();

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

    pub fn get_mob(&self, id: u32) -> Mob {
        let found = self.mobs
            .iter()
            .find(|p| p.id == id);

        found.unwrap().clone()
    }

    pub fn get_player(&self, login: &String) -> Player {
        let found = self.players
            .iter()
            .find(|p| p.login.eq(login));

        found.unwrap().clone()
    }

    pub fn update_mob(&mut self, mob: Mob) {
        let index = self.mobs.iter().position(|x| x.id == mob.id).unwrap();
        self.mobs.insert(index, mob);
    }
}


impl Game {
    fn next_mob_id(&mut self) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id += 1;
        id
    }
}
