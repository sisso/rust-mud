use super::game::*;
use std::collections::HashMap;
use super::view_login;
use super::view_mainloop;

struct PlayerState {
    id: u32,
    login: Option<String>,
}

pub struct GameController {
    game: Game,
    players: HashMap<u32, PlayerState>,
}

// TODO: move login and input handling to utility
impl GameController {
    pub fn new(game: Game) -> Self {
        GameController {
            game,
            players: HashMap::new()
        }
    }

    pub fn handle(&mut self, connects: Vec<u32>, disconnects: Vec<u32>, inputs: Vec<(u32, String)>) -> Vec<(u32, String)> {
        let mut outputs: Vec<(u32, String)> = vec![];

        // handle new players
        for id in connects {
            self.players.insert(id, PlayerState {
                id,
                login: None
            });

            let out = view_login::handle_welcome();
            outputs.push((id, out));
        }

        // handle disconnected players
        for id in disconnects {
            self.game.player_disconnect(id);
        }

        // handle players inputs
        for (id, input) in inputs {
            let maybe_login = {
                let player  = self.players.get(&id).unwrap();
                // TODO: remove clone?
                player.login.clone()
            };


            if let Some(login) = maybe_login {
                let output = view_mainloop::handle(&mut self.game, &login, input);
                outputs.push((id, output));
            } else {
                let out = match view_login::handle(input) {
                    (Some(login), out) => {
                        let player = self.players.entry(id);
                        player.and_modify(|player| {
                            player.login = Some(login.clone());
                        });

                        // TODO: externalize avatar creation

                        // search initial room
                        let rooms = self.game.get_rooms_by_tag(&RoomTag::INITIAL);
                        let inital_room_id = rooms.first().unwrap();

                        // add player avatar
                        let mut mob = self.game.new_mob(*inital_room_id, format!("char-{}", login));
                        mob.tags.insert(MobTag::AVATAR);
                        let mob_id = mob.id;
                        self.game.add_mob(mob);

                        // add player to game
                        self.game.player_connect(id, &login, mob_id);

                        let look_output = view_mainloop::handle_look(&self.game, &login);

                        format!("{}{}", out, look_output)
                    },
                    (_, out) => out,
                };

                outputs.push((id, out));
            }
        }

        outputs
    }
}
