#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct PlayerId(pub u32);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PlayerId({})", self.0)
    }
}

pub struct Container {
    next_mob_id: u32,
    next_player_id: u32,
    rooms: Vec<Room>,
    mobs: Vec<Mob>,
    players: Vec<Player>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            next_mob_id: 0,
            next_player_id: 0,
            rooms: vec![],
            mobs: vec![],
            players: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub login: String,
    pub avatar_id: u32
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u32,
    pub room_id: u32,
    pub label: String,
    pub is_avatar: bool
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dir {
    N,
    S,
    W,
    E
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::E,
            Dir::W => Dir::W,
        }
    }
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

#[derive(Clone, Debug)]
pub struct Room {
    pub id: u32,
    pub label: String,
    pub desc: String,
    pub exits: Vec<(Dir, u32)>,
}


pub struct PlayerCtx<'a> {
    pub player: &'a Player,
    pub avatar: &'a Mob,
    pub room: &'a Room,
}

impl Container {
    pub fn list_players(&self) -> Vec<&PlayerId> {
        self.players.iter().map(|i| &i.id).collect()
    }

    pub fn player_connect(&mut self, login: String, avatar_id: u32) -> &Player {
        let id = PlayerId(self.next_player_id());

        println!("game - adding player {}/{}", id, login);

        let player = Player {
            id,
            login: login,
            avatar_id,
        };

        self.players.push(player);

        &self.players.last().unwrap()
    }

    pub fn player_disconnect(&mut self, id: &PlayerId) {
        println!("game - removing player {}", id);

        let index = self.players.iter().position(|x| x.id == *id).unwrap();
        self.players.remove(index);
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.push(room);
    }

//    pub fn get_rooms_by_tag(&self, tag: &RoomTag) -> Vec<u32> {
//        self.rooms
//            .iter()
//            .filter(|room| room.tags.contains(tag))
//            .map(|room| room.id)
//            .collect()
//    }

    pub fn get_room(&self, id: &u32) -> &Room {
        let room = self.rooms
            .iter()
            .find(|room| { room.id == *id })
            .unwrap();

        room
    }

    pub fn add_mob(&mut self, mob: Mob) -> &Mob {
        self.mobs.push(mob);
        self.mobs.last().unwrap()
    }

    pub fn get_mob(&self, id: &u32) -> &Mob {
        let found = self.mobs
            .iter()
            .find(|p| p.id == *id);

        found.unwrap()
    }

    pub fn get_player_by_login(&self, login: &String) -> &Player {
        let found = self.players
            .iter()
            .find(|p| p.login.eq(login));

        found.expect(format!("player with login {} not found", login).as_str())
    }

    pub fn get_avatar(&self, player_id: &PlayerId) -> &Mob {
        let player = self.get_player_by_id(player_id);
        self.get_mob(&player.avatar_id)
    }

    pub fn get_player_by_id(&self, id: &PlayerId) -> &Player {
        let found = self.players
            .iter()
            .find(|p| p.id == *id);

        found.expect(format!("player with login {} not found", id).as_str())
    }

    pub fn update_mob(&mut self, mob: Mob) {
        let index = self.mobs.iter().position(|x| x.id == mob.id).unwrap();
        self.mobs.insert(index, mob);
    }

    pub fn next_mob_id(&mut self) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id += 1;
        id
    }

    pub fn get_player_context(&self, player_id: &PlayerId) -> PlayerCtx {
        let player = self.get_player_by_id(player_id);
        let mob = self.get_mob(&player.avatar_id);
        let room = self.get_room(&mob.room_id);

        PlayerCtx {
            player,
            avatar: mob,
            room
        }
    }

}


impl Container {
    fn next_player_id(&mut self) -> u32 {
        let id = self.next_player_id;
        self.next_player_id += 1;
        id
    }
}
