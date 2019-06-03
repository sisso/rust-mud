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

pub struct HandleOutput {
    pub player_id: u32,
    pub room_id: Vec<u32>,
    pub player_msg: Vec<String>,
    pub room_msg: Vec<String>
}

impl HandleOutput {
    fn private(player_id: u32, msg: String) -> Self {
        HandleOutput {
            player_id: player_id,
            room_id: vec![],
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
            players: HashMap::new()
        }
    }

    pub fn handle(&mut self, connects: Vec<u32>, disconnects: Vec<u32>, inputs: Vec<(u32, String)>) -> Vec<HandleOutput> {
        let mut outputs = vec![];

        // handle new players
        for id in connects {
            println!("gamecontroller - {} receive new player", id);
            self.players.insert(id, PlayerState {
                id,
                login: None
            });

            let out = view_login::handle_welcome();
            outputs.push(HandleOutput::private(id, out));
        }

        // handle disconnected players
        for id in disconnects {
            println!("gamecontroller - {} removing player", id);
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
                println!("gamecontroller - {} handling login input '{}'", id, input);
                let out = view_mainloop::handle(&mut self.game, &login, input);
                outputs.push(HandleOutput::private(id, out));
            } else {
                println!("gamecontroller - {} handling login '{}'", id, input);

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

                outputs.push(HandleOutput::private(id, out));
            }
        }

        outputs
    }
}
