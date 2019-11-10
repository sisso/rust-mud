use commons::*;
use std::collections::HashMap;

use super::container::Container;
use super::mob::*;
use super::mob::MobId;

use logs::*;
use crate::game::labels::Label;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub login: String,
    pub mob_id: MobId
}



pub struct PlayerRepository {
    index: HashMap<PlayerId, Player>
}

impl PlayerRepository {
    pub fn new() -> PlayerRepository {
        PlayerRepository {
            index: HashMap::new(),
        }
    }

    pub fn list_players(&self) -> Vec<PlayerId> {
        self.index
            .iter()
            .into_iter()
            .map(| (id, _)| *id)
            .collect()
    }

    pub fn login(&self, login: &str) -> Option<PlayerId> {
        self.index.iter().find_map(|(id, player)| {
            if player.login.eq(login) {
                Some(*id)
            } else {
                None
            }
        })
    }

    pub fn create(&mut self, player_id: PlayerId, login: String, avatar_id: MobId) -> &Player {
        info!("creating player {:?}/{}", player_id, login);

        let player = Player {
            id: player_id,
            login: login,
            mob_id: avatar_id,
        };

        self.index.insert(player_id, player);
        self.index.get(&player_id).unwrap()
    }

   pub fn find_player_from_avatar_mob_id(&self, mob_id: MobId) -> Option<&Player> {
        self.index
            .iter()
            .find(|(_, p)| p.mob_id == mob_id)
            .map(|(_, player)| player)
    }

    pub fn find_player_id_from_avatar_mob_id(&self, mob_id: MobId) -> Option<PlayerId> {
        self.index
            .iter()
            .find(|(_, p)| p.mob_id == mob_id)
            .map(|(id, _)| id.clone())
    }

    pub fn get_player_by_id(&self, id: PlayerId) -> &Player {
        self.index
            .iter()
            .find(|(pid, _)| **pid == id)
            .map(|(_, p)| p)
            .expect(format!("player with id {:?} not found", id).as_str())
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        use serde_json::json;
//
//        for (player_id, player) in self.index.iter() {
//            save.add(player_id.0, "player", json!({
//                "mob_id": player.avatar_id.0,
//                "login": player.login
//            }));
//        }
//    }
}
