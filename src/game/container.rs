use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;
use super::item::*;

use crate::utils::*;

pub struct Container {
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    pub items: ItemRepository,
    pub rooms: RoomRepository,
    tick: Tick,
    time: Seconds,
    next_player_id: u32,
    spawns: Vec<Spawn>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            players: PlayerRepository::new(),
            mobs: MobRepository::new(),
            items: ItemRepository::new(),
            rooms: RoomRepository::new(),
            tick: Tick(0),
            time: Seconds(0.0),
            next_player_id: 0,
            spawns: vec![],
        }
    }

    pub fn get_player_context(&self, player_id: &PlayerId) -> PlayerCtx {
        let player = self.players.get_player_by_id(player_id);
        let mob = self.mobs.get(&player.avatar_id);
        let room = self.rooms.get(&mob.room_id);

        PlayerCtx {
            player,
            avatar: mob,
            room
        }
    }

    pub fn add_spawn(&mut self, spawn: Spawn) {
        self.spawns.push(spawn);
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

    pub fn instantiate_item(&mut self, item_prefab_id: ItemPrefabId) -> Item {
        let item_id = self.items.next_item_id();
        let prefab = self.items.get_prefab(&item_prefab_id);

        let mut item = Item::new(
            item_id,
            prefab.typ,
            prefab.label.clone()
        );

        item.amount = prefab.amount;
        item.item_def_id = Some(item_prefab_id);

        item
    }

    pub fn instantiate(&mut self, mob_prefab_id: MobPrefabId, room_id: RoomId) -> &Mob {
        let prefab = self.mobs.get_mob_prefab(&mob_prefab_id).clone();

        // create mob
        let mob_id = self.mobs.new_id();

        // add items
        let items = prefab.inventory;
        for item_prefab_id in items {
            let item = self.instantiate_item(item_prefab_id);
            self.items.add(item, ItemLocation::Mob { mob_id });
        }

        // instantiate
        let mob = Mob::new(mob_id, room_id, prefab.label, prefab.attributes);
        self.mobs.add(mob);
        self.mobs.get(&mob_id)
    }
}

impl Container {
    fn next_player_id(&mut self) -> u32 {
        let id = self.next_player_id;
        self.next_player_id += 1;
        id
    }
}
