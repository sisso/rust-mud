use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;

pub struct Container {
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    tick: Tick,
    time: Seconds,
    next_mob_id: u32,
    next_player_id: u32,
    rooms: Vec<Room>,
    spawns: Vec<Spawn>,
    mob_prefabs: Vec<MobPrefab>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            players: PlayerRepository::new(),
            mobs: MobRepository::new(),
            tick: Tick(0),
            time: Seconds(0.0),
            next_mob_id: 0,
            next_player_id: 0,
            rooms: vec![],
            spawns: vec![],
            mob_prefabs: vec![],
        }
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

    pub fn get_player_context(&self, player_id: &PlayerId) -> PlayerCtx {
        let player = self.players.get_player_by_id(player_id);
        let mob = self.mobs.get(&player.avatar_id);
        let room = self.get_room(&mob.room_id);

        PlayerCtx {
            player,
            avatar: mob,
            room
        }
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
