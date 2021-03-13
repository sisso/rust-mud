use super::mob::MobId;
use crate::errors::{Error, Result};
use commons::*;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub login: String,
    pub mob_id: MobId,
}

#[derive(Debug, Clone)]
pub struct PlayerRepository {
    index: HashMap<PlayerId, Player>,
}

impl PlayerRepository {
    pub fn new() -> PlayerRepository {
        PlayerRepository {
            index: HashMap::new(),
        }
    }

    pub fn list_players(&self) -> Vec<PlayerId> {
        self.index.keys().copied().collect()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &'a Player> + 'a {
        self.index.values()
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

    pub fn find_from_mob(&self, mob_id: MobId) -> Option<PlayerId> {
        self.index
            .iter()
            .find(|(_, p)| p.mob_id == mob_id)
            .map(|(&player_id, _)| player_id)
    }

    pub fn get(&self, id: PlayerId) -> Option<&Player> {
        self.index
            .iter()
            .find(|(pid, _)| **pid == id)
            .map(|(_, p)| p)
    }

    pub fn get_mob(&self, player_id: PlayerId) -> Option<MobId> {
        self.index.get(&player_id).map(|player| player.mob_id)
    }

    pub fn set_mob(&mut self, player_id: PlayerId, mob_id: MobId) -> Result<()> {
        self.index
            .get_mut(&player_id)
            .map(|player| {
                player.mob_id = mob_id;
                ()
            })
            .ok_or(Error::NotFoundFailure)
    }
}
