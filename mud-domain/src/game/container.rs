use super::mob::*;
use super::player::*;
use super::room::*;
use super::spawn::*;
use super::domain::*;
use super::item::*;

use crate::game::body::create_body;
use commons::PlayerId;
use crate::game::obj::Objects;

pub struct Container {
    pub time: GameTime,
    pub objects: Objects,
    pub players: PlayerRepository,
    pub mobs: MobRepository,
    pub items: ItemRepository,
    pub rooms: RoomRepository,
    pub spawns: Spawns,
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
        }
    }

    /// If mob have no room, a exception will be throw
    pub fn get_player_context(&self, player_id: PlayerId) -> PlayerCtx {
        let player = self.players.get_player_by_id(player_id);
        let mob = self.mobs.get(player.avatar_id);
        let room = self.rooms.get(&mob.room_id.unwrap());

        PlayerCtx {
            player,
            avatar: mob,
            room
        }
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        self.players.save(save);
//        self.mobs.save(save);
//        self.items.save(save);
//    }
}

