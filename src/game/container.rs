use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;

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

    pub fn player_connect(&mut self, login: String, avatar_id: MobId) -> &Player {
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

    pub fn get_room(&self, id: &RoomId) -> &Room {
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

    pub fn get_mob(&self, id: &MobId) -> &Mob {
        let found = self.mobs
            .iter()
            .find(|p| p.id == *id);

        found.unwrap()
    }

    pub fn find_mob(&self, id: &MobId) -> Option<&Mob> {
        self.mobs
            .iter()
            .find(|p| p.id == *id)
    }

    pub fn is_mob(&self, id: &MobId) -> bool {
        self.mobs
            .iter()
            .find(|p| p.id == *id)
            .is_some()
    }

    pub fn get_mobs(&self) -> Vec<MobId> {
        self.mobs.iter().map(|i| i.id).collect()
    }

    pub fn get_mob_mut(&mut self, id: &MobId) -> &mut Mob {
        let found = self.mobs
            .iter_mut()
            .find(|p| p.id == *id);

        found.unwrap()
    }

    pub fn remove_mob(&mut self, id: &MobId) {
        let index = self.mobs.iter().position(|x| x.id == *id).unwrap();
        self.mobs.remove(index);
    }

    pub fn find_player_from_avatar_mob_id(&self, mob_id: &MobId) -> Option<&Player> {
        self.players
            .iter()
            .find(|p| p.avatar_id == *mob_id)
    }

    pub fn find_player_id_from_avatar_mob_id(&self, mob_id: &MobId) -> Option<PlayerId> {
        self.players
            .iter()
            .find(|p| p.avatar_id == *mob_id)
            .map(|i| i.id.clone())
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

    pub fn next_mob_id(&mut self) -> MobId {
        let id = self.next_mob_id;
        self.next_mob_id += 1;
        MobId(id)
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

    pub fn find_mobs_at(&self, room_id: &RoomId) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|i| i.room_id == *room_id)
            .collect()
    }

    pub fn search_mob_by_name_at(&self, room_id: &RoomId, query: &String) -> Vec<&Mob> {
        self.mobs
            .iter()
            .filter(|i| i.room_id == *room_id)
            .filter(|i| i.label.eq(query))
            .collect()
    }

    pub fn set_mob_kill_target(&mut self, mob_id: &MobId, target: &MobId) {
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
            .expect(format!("could not found mob prefab id {:?}", id).as_str())
    }

    pub fn list_spawn(&self) -> Vec<SpawnId> {
        self.spawns.iter().map(|i| i.id.clone()).collect()
    }

    pub fn get_spawn_by_id_mut(&mut self, spawn_id: &SpawnId) -> &mut Spawn {
        self.spawns
            .iter_mut()
            .find(|i| i.id == *spawn_id)
            .expect("could not find")
    }

    pub fn get_spawn_by_id(&self, spawn_id: &SpawnId) -> &Spawn {
        self.spawns
            .iter()
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
