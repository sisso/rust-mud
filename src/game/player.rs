use std::collections::HashMap;

use super::container::Container;
use super::domain::*;
use super::mob::*;
use super::mob::MobId;

use crate::utils::*;
use crate::utils::save::Save;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct PlayerId(pub u32);

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PlayerId({})", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: PlayerId,
    pub login: String,
    pub avatar_id: MobId
}

pub fn add_player(container: &mut Container, login: &String) -> PlayerId {
    // add player avatar
    let mob_id = container.mobs.new_id();

    let mut mob = Mob::new(
        mob_id,
        super::mob::INITIAL_ROOM_ID,
        login.clone(),
        Attributes {
            attack: 12,
            defense: 12,
            damage: Damage {
                min: 1,
                max: 4,
            },
            pv: Pv {
                current: 10,
                max: 10,
                heal_rate: Second(1.0),
            },
            attack_calm_down: Second(1.0)
        }
    );
    mob.is_avatar = true;

    container.mobs.add(mob);

    // add player to game
    let player = container.players.player_connect(login.clone(), mob_id);
    player.id
}

pub struct PlayerRepository {
    next_id: NextId,
    index: HashMap<PlayerId, Player>
}

impl PlayerRepository {
    pub fn new() -> PlayerRepository {
        PlayerRepository {
            next_id: NextId::new(),
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

    pub fn player_connect(&mut self, login: String, avatar_id: MobId) -> &Player {
        let id = PlayerId(self.next_id.next());

        info!("game - adding player {}/{}", id, login);

        let player = Player {
            id,
            login: login,
            avatar_id,
        };

        self.index.insert(id, player);
        self.index.get(&id).unwrap()
    }

    pub fn player_disconnect(&mut self, id: PlayerId) {
        info!("game - removing player {}", id);
        self.index.remove(&id);
    }

    pub fn find_player_from_avatar_mob_id(&self, mob_id: MobId) -> Option<&Player> {
        self.index
            .iter()
            .find(|(_, p)| p.avatar_id == mob_id)
            .map(|(_, player)| player)
    }

    pub fn find_player_id_from_avatar_mob_id(&self, mob_id: &MobId) -> Option<PlayerId> {
        self.index
            .iter()
            .find(|(_, p)| p.avatar_id == *mob_id)
            .map(|(id, _)| id.clone())
    }

    pub fn get_player_by_id(&self, id: PlayerId) -> &Player {
        self.index
            .iter()
            .find(|(pid, _)| **pid == id)
            .map(|(_, p)| p)
            .expect(format!("player with id {} not found", id).as_str())
    }

    pub fn save(&self, save: &mut dyn Save) {
        use serde_json::json;

        for (player_id, player) in self.index.iter() {
            save.add(player_id.0, "player", json!({
                "mob_id": player.avatar_id.0,
                "login": player.login
            }));
        }
    }
}
