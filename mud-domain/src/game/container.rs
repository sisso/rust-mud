use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;
use super::item::*;

use crate::game::body::create_body;
use commons::PlayerId;

pub struct Container {
    pub time: GameTime,
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    pub items: ItemRepository,
    pub rooms: RoomRepository,
    spawns: Vec<Spawn>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            time: GameTime::new(),
            players: PlayerRepository::new(),
            mobs: MobRepository::new(),
            items: ItemRepository::new(),
            rooms: RoomRepository::new(),
            spawns: vec![],
        }
    }

    pub fn get_player_context(&self, player_id: PlayerId) -> PlayerCtx {
        let player = self.players.get_player_by_id(player_id);
        let mob = self.mobs.get(player.avatar_id);
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

    // TODO: move outside of container
    pub fn instantiate(&mut self, mob_prefab_id: MobPrefabId, room_id: RoomId) -> &Mob {
        let prefab = self.mobs.get_mob_prefab(&mob_prefab_id).clone();

        // create mob
        let mob_id = self.mobs.new_id();

        // add items
        let items = prefab.inventory;
        for item_prefab_id in items {
            self.items.instantiate_item(item_prefab_id, ItemLocation::Mob { mob_id });
        }

        // instantiate
        let mob = Mob::new(mob_id, room_id, prefab.label, prefab.attributes);
        self.mobs.add(mob);
        self.mobs.get(mob_id)
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        self.players.save(save);
//        self.mobs.save(save);
//        self.items.save(save);
//    }
}
