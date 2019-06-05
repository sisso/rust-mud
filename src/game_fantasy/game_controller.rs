use super::game::*;
use std::collections::{HashMap, HashSet};
use super::view_login;
use super::view_mainloop;

struct PlayerState {
    connection_id: u32,
    player_id: Option<u32>,
    login: Option<String>,
}

pub struct GameController {
    game: Game,
    players: Vec<PlayerState>,
}

pub struct HandleOutput {
    pub player_id: u32,
    pub room_id: Option<u32>,
    pub player_msg: Vec<String>,
    pub room_msg: Vec<String>
}

impl HandleOutput {
    pub fn private(player_id: u32, msg: String) -> Self {
        HandleOutput {
            player_id: player_id,
            room_id: None,
            player_msg: vec![msg],
            room_msg: vec![]
        }
    }
}

// TODO: move login and input handling to utility
impl GameController {
    pub fn new(game: Game) -> Self {
        GameController {
            game,
            players: vec![]
        }
    }

    pub fn connection_id_from_player_id(&self, player_id: &u32) -> u32 {
        self.players
            .iter()
            // TODO: option.contains?
            .find(|i| i.player_id == Some(*player_id))
            .map(|state| state.connection_id)
            .unwrap()
    }

    pub fn players_per_room(&self) -> HashMap<u32, Vec<u32>> {
        let room_player: Vec<(u32, u32)> =
            self.game.list_players()
                .into_iter()
                .map(|player_id| {
                    let player = self.game.get_player_by_id(&player_id);
                    let avatar = self.game.get_mob(player.avatar_id);
                    (avatar.room_id, player_id)
                })
                .collect();

        // group_by
        let mut result: HashMap<u32, Vec<u32>> = HashMap::new();
        for (room_id, player_id) in room_player {
            result.entry(room_id).or_insert(vec![]).push(player_id);
        }
        result
    }

    pub fn handle(&mut self, connects: Vec<u32>, disconnects: Vec<u32>, inputs: Vec<(u32, String)>) -> Vec<HandleOutput> {
        let mut outputs = vec![];

        // handle new players
        for id in connects {
            println!("gamecontroller - {} receive new player", id);
            self.players.push(PlayerState {
                connection_id: id,
                login: None,
                player_id: None,
            });

            let out = view_login::handle_welcome();
            outputs.push(HandleOutput::private(id, out));
        }

        // handle disconnected players
        for id in disconnects {
            println!("gamecontroller - {} removing player", id);
            self.game.player_disconnect(&id);
            let index = self.players.iter().position(|i| i.connection_id == id).unwrap();
            let _ = self.players.remove(index);
        }

        // handle players inputs
        for (id, input) in inputs {
            let maybe_login = {
                let player  = self.players.iter().find(|i| i.connection_id == id).unwrap();
                // TODO: remove clone?
                player.login.clone()
            };


            if let Some(login) = maybe_login {
                println!("gamecontroller - {} handling input '{}'", id, input);
                let mut out = view_mainloop::handle(&mut self.game, id, &login, input);
                outputs.append(&mut out);
            } else {
                println!("gamecontroller - {} handling login '{}'", id, input);

                let out = match view_login::handle(input) {
                    (Some(login), out) => {
                        let index = self.players.iter().position(|i| i.connection_id == id).unwrap();

                        // TODO: externalize avatar creation
                        // search initial room
                        let rooms = self.game.get_rooms_by_tag(&RoomTag::INITIAL);
                        let inital_room_id = rooms.first().unwrap();

                        // add player avatar
                        let tags = vec![MobTag::AVATAR].iter().cloned().collect();
                        let mob = self.game.new_mob(inital_room_id, login.clone(), tags);
                        let mob_id = mob.id;

                        // add player to game
                        let player = self.game.player_connect(login.clone(), mob_id);

                        // update local state
                        let connection_state = self.players.get_mut(index).unwrap();
                        connection_state.login = Some(login.clone());
                        connection_state.player_id = Some(player.id);

                        let look_output = view_mainloop::handle_look(&self.game, &login);

                        format!("{}{}", out, look_output)
                    },
                    (_, out) => out,
                };

                outputs.push(HandleOutput::private(id, out));
            }
        }

        outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_players_per_room() {
        let mut game = Game::new();

        game.add_room(Room {
            id: 0,
            label: "room1".to_string(),
            desc: "".to_string(),
            exits: vec![],
            tags: HashSet::new()
        });

        let mob_player_0_id = game.new_mob(&0, "sisso".to_string(), HashSet::new()).id;
        let mob_player_1_id = game.new_mob(&0, "abibue".to_string(), HashSet::new()).id;

        game.player_connect("sisso".to_string(), mob_player_0_id);
        game.player_connect( "abibue".to_string(), mob_player_1_id);

        let mut gc = GameController::new(game);

        let map = gc.players_per_room();
        let result = map.get(&0);
        println!("{:?}", result);
        assert_eq!(result, Some(&vec![0, 1]));
    }
}
