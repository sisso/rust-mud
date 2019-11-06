use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;
use super::item::*;

use commons::{PlayerId, ObjId};
use crate::game::obj::Objects;
use crate::game::location::Locations;
use crate::game::equip::{Equips};

pub struct Container {
    pub time: GameTime,
    pub objects: Objects,
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    pub items: ItemRepository,
    pub rooms: RoomRepository,
    pub spawns: Spawns,
    pub locations: Locations,
    pub equips: Equips,
}

impl Container {
    pub fn new() -> Self {
        Container {
            time: GameTime::new(),
            objects: Objects::new(),
            players: PlayerRepository::new(),
            mobs: MobRepository::new(),
            items: ItemRepository::new(),
            rooms: RoomRepository::new(),
            spawns: Spawns::new(),
            locations: Locations::new(),
            equips: Equips::new(),
        }
    }

    pub fn remove(&mut self, obj_id: ObjId) {
        self.mobs.remove(obj_id);
        self.items.remove(obj_id);
        self.locations.remove(obj_id);
        // self.rooms.remove(obj_id);
        // self.spanws.remove(obj_id);
        self.equips.remove(obj_id);
        self.objects.remove(obj_id);
    }

    // TODO: add Result or complete remove this method
    /// If mob have no room, a exception will be throw
    pub fn get_player_context(&self, player_id: PlayerId) -> PlayerCtx {
        let player = self.players.get_player_by_id(player_id);
        let mob = self.mobs.get(player.mob_id).unwrap();
        let room_id = self.locations.get(mob.id).unwrap();
        let room = self.rooms.get(room_id).unwrap();

        PlayerCtx {
            player,
            mob,
            room
        }
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        self.players.save(save);
//        self.mobs.save(save);
//        self.items.save(save);
//    }
}

