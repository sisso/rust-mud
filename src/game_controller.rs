use crate::game::*;
use std::collections::HashMap;
use crate::view_login;
use crate::view_mainloop;
use core::borrow::Borrow;

struct PlayerState {
    id: u32,
    login: Option<String>,
}

pub struct GameController {
    game: Game,
    players: HashMap<u32, PlayerState>,
}

impl GameController {
    pub fn new(game: Game) -> Self {
        GameController {
            game,
            players: HashMap::new()
        }
    }

    pub fn handle(&mut self, connections_id: Vec<u32>, inputs: Vec<(u32, String)>) -> Vec<(u32, String)> {
        let mut outputs: Vec<(u32, String)> = vec![];

        // handle new players
        for id in connections_id {
            if !self.players.contains_key(& id) {
                self.players.insert(id, PlayerState {
                    id,
                    login: None
                });

                let out = view_login::handle_welcome(id);
                outputs.push((id, out));
            }
        }

        // handle players inputs
        for (id, input) in inputs {
            let maybe_login = {
                let player = self.players.get(&id).unwrap();
                // TODO: remove clone?
                player.login.clone()
            };


            if let Some(login) = maybe_login {
                if let Some(out) = view_mainloop::handle(&login, input) {
                    outputs.push((id, out));
                };
            } else {
                let out = match view_login::handle(id, input) {
                    (Some(login), out) => {
                        let player = self.players.entry(id);
                        player.and_modify(|player| {
                            player.login = Some(login);
                        });
                        out
                    },
                    (_, out) => out,
                };

                outputs.push((id, out));
            }
        }

        outputs
    }
}
