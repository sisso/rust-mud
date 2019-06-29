use super::spawn::*;
use super::mob::*;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct Tick(u32);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct PlayerId(pub u32);

#[derive(Clone,Copy,Debug)]
pub struct Seconds(pub f32);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct RoomId(pub u32);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PlayerId({})", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub login: String,
    pub avatar_id: u32
}


#[derive(Clone, Debug)]
pub struct MobPrefab {
    pub id: MobPrefabId,
    pub label: String,
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

pub struct Container {
    tick: Tick,
    time: Seconds,
    next_mob_id: u32,
    next_player_id: u32,
    rooms: Vec<Room>,
    mobs: Vec<Mob>,
    players: Vec<Player>,
    spawns: Vec<Spawn>,
    mob_prefabs: Vec<MobPrefab>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            tick: Tick(0),
            time: Seconds(0.0),
            next_mob_id: 0,
            next_player_id: 0,
            rooms: vec![],
            mobs: vec![],
            players: vec![],
            spawns: vec![],
            mob_prefabs: vec![],
        }
    }

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

    pub fn get_mobs(&self) -> Vec<MobId> {
        self.mobs.iter().map(|i| MobId(i.id)).collect()
    }

    fn get_mob_mut(&mut self, id: &u32) -> &mut Mob {
        let found = self.mobs
            .iter_mut()
            .find(|p| p.id == *id);

        found.unwrap()
    }

//    pub fn get_player_by_login(&self, login: &String) -> &Player {
//        let found = self.players
//            .iter()
//            .find(|p| p.login.eq(login));
//
//        found.expect(format!("player with login {} not found", login).as_str())
//    }

    pub fn find_player_from_avatar_mob_id(&self, mob_id: &MobId) -> Option<&Player> {
        self.players
            .iter()
            .find(|p| p.avatar_id == mob_id.0)
    }

    pub fn get_player_by_id(&self, id: &PlayerId) -> &Player {
        let found = self.players
            .iter()
            .find(|p| p.id == *id);

        found.expect(format!("player with login {} not found", id).as_str())
    }

    pub fn update_mob(&mut self, mob: Mob) {
        let index = self.mobs.iter().position(|x| x.id == mob.id).unwrap();
        self.mobs[index] = mob;
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

    pub fn find_mobs_at(&self, room_id: &u32) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|i| i.room_id == *room_id)
            .collect()
    }

    pub fn search_mob_by_name_at(&self, room_id: &RoomId, query: &String) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|i| i.room_id == room_id.0)
            .filter(|i| i.label.eq(query))
            .collect()
    }

    pub fn set_mob_kill_target(&mut self, mob_id: &u32, target: &MobId) {
        let mob = self.get_mob_mut(mob_id);
        mob.command = MobCommand::Kill { target: target.clone() };
    }

    pub fn add_spawn(&mut self, spawn: Spawn) {
        self.spawns.push(spawn);
    }

    pub fn add_mob_prefab(&mut self, mob_prefab: MobPrefab) {
        self.mob_prefabs.push(mob_prefab);
    }

    pub fn get_mob_prefab(&mut self, id: &MobPrefabId) -> &MobPrefab{
        self.mob_prefabs
            .iter()
            .find(|i| i.id == *id)
            .expect("could not found mob prefab")
    }

    pub fn list_spawn(&self) -> Vec<SpawnId> {
        self.spawns.iter().map(|i| i.id.clone()).collect()
    }

    pub fn get_spawn_by_id(&mut self, spawn_id: &SpawnId) -> &mut Spawn {
        self.spawns
            .iter_mut()
            .find(|i| i.id == *spawn_id)
            .expect("could not find")
    }

    pub fn update_tick(&mut self, delta_time: Seconds) {
        self.tick = Tick(self.tick.0 + 1);
        self.time = Seconds(self.time.0 + delta_time.0);
//        println!("container {:?}/{:?}", self.tick, self.time);
    }

//    pub fn get_tick(&self) -> &Tick {
//        &self.tick
//    }

    pub fn get_time(&self) -> &Seconds {
        &self.time
    }
}


impl Container {
    fn next_player_id(&mut self) -> u32 {
        let id = self.next_player_id;
        self.next_player_id += 1;
        id
    }
}
