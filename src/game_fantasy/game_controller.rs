use super::game::*;
use crate::server;
use crate::server::ConnectionId;
use std::collections::{HashMap};
use super::view_login;
use super::view_mainloop;

struct PlayerState {
    connection_id: ConnectionId,
    player_id: Option<PlayerId>,
    login: Option<String>,
}

pub struct GameController {
    game: Game,
    players: Vec<PlayerState>,
}

// TODO: split player and room?
pub struct HandleOutput {
    pub player_id: PlayerId,
    pub room_id: Option<u32>,
    pub player_msg: Vec<String>,
    pub room_msg: Vec<String>
}

impl HandleOutput {
    pub fn private(player_id: PlayerId, msg: String) -> Self {
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

    pub fn connection_id_from_player_id(&self, player_id: &PlayerId) -> ConnectionId {
        self.players
            .iter()
            // TODO: option.contains?
            .find(|i| i.player_id == Some(*player_id))
            .map(|state| state.connection_id)
            .unwrap()
    }

    pub fn players_per_room(&self) -> HashMap<u32, Vec<PlayerId>> {
        let room_player: Vec<(u32, PlayerId)> =
            self.game.list_players()
                .into_iter()
                .map(|player_id| {
                    let player = self.game.get_player_by_id(player_id);
                    let avatar = self.game.get_mob(player.avatar_id);
                    (avatar.room_id, *player_id)
                })
                .collect();

        // group_by
        let mut result: HashMap<u32, Vec<PlayerId>> = HashMap::new();
        for (room_id, player_id) in room_player {
            result.entry(room_id).or_insert(vec![]).push(player_id);
        }
        result
    }

    pub fn player_id_from_connection_id(&self, connection: &ConnectionId) -> PlayerId {
        self.players
            .iter()
            .find(|i| i.connection_id == *connection)
            .map(|i| i.player_id.unwrap())
            .unwrap()
    }

    pub fn handle(&mut self, connects: Vec<ConnectionId>, disconnects: Vec<ConnectionId>, inputs: Vec<(ConnectionId, String)>) -> Vec<server::Output> {
        let mut outputs = vec![];

        // handle new players
        for connection in connects {
            println!("gamecontroller - {} receive new player", connection.id);
            self.players.push(PlayerState {
                connection_id: connection,
                login: None,
                player_id: None,
            });

            let out = view_login::handle_welcome();
//            self.append_output(&mut outputs, HandleOutput::private(player_id, out));
            outputs.push(server::Output {
                dest_connections_id: vec![connection],
                output: out,
            });
        }

        // handle disconnected players
        for connection in disconnects {
            let index = self.players.iter().position(|i| i.connection_id == connection).unwrap();
            let connected = self.players.get(index).unwrap();

            match connected.player_id {
                Some(player_id) => {
                    println!("gamecontroller - {} removing player {}", connection.id, player_id);
                    self.game.player_disconnect(&player_id);
                }
                _ => {
                    println!("gamecontroller - {} removing non loged player", connection.id);
                }
            }

            let _ = self.players.remove(index);
        }

        // handle players inputs
        for (connection, input) in inputs {
            let maybe_login = {
                let player  = self.players.iter().find(|i| i.connection_id == connection).unwrap();
                // TODO: remove clone?
                player.login.clone()
            };

            if let Some(login) = maybe_login {
                println!("gamecontroller - {} handling input '{}'", connection.id, input);

                let player_id = self.player_id_from_connection_id(&connection);
                let out = view_mainloop::handle(&mut self.game, &player_id, &login, input);
                self.append_outputs(&mut outputs, out);
            } else {
                println!("gamecontroller - {} handling login '{}'", connection.id, input);

                let out = match view_login::handle(input) {
                    (Some(login), out) => {
                        let index = self.players.iter().position(|i| i.connection_id == connection).unwrap();

                        // TODO: externalize avatar creation

                        // add player avatar
                        let mob_id = self.game.next_mob_id();

                        let mob = Mob {
                            id: mob_id,
                            label: login.clone(),
                            room_id: 0,
                            is_avatar: true
                        };
                        self.game.add_mob(mob);

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

                outputs.push(server::Output {
                    dest_connections_id: vec![connection],
                    output: out,
                });
            }
        }

        outputs
    }

    fn append_output(&self, output: &mut Vec<server::Output>, handle_output: HandleOutput) {
        let connection_id = self.connection_id_from_player_id(&handle_output.player_id);

        for msg in handle_output.player_msg {
            output.push(server::Output {
                dest_connections_id: vec![connection_id],
                output: msg
            })
        }

        if let Some(room_id) = handle_output.room_id {
            // TODO: cache outside
            let players_per_room = self.players_per_room();
            if let Some(players) = players_per_room.get(&room_id) {
                let connections_id: Vec<ConnectionId> =
                    players
                        .iter()
                        .map(|player_id| self.connection_id_from_player_id(&player_id))
                        .filter(|other_connection_id| *other_connection_id != connection_id)
                        .collect();

                for msg in handle_output.room_msg {
                    output.push(server::Output {
                        dest_connections_id: connections_id.clone(),
                        output: msg
                    });
                }
            }
        }
    }

    fn append_outputs(&self, output: &mut Vec<server::Output>, handle_output: Vec<HandleOutput>) {
        for i in handle_output {
            self.append_output(output, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r_mob(game: &mut Game, label: String) -> &Mob {
        let mob_id = game.next_mob_id();

        let mob = Mob {
            id: mob_id,
            label: label,
            room_id: 0,
            is_avatar: true
        };

        game.add_mob(mob)
    }

    #[test]
    fn test_players_per_room() {
        let mut game = Game::new();

        game.add_room(Room {
            id: 0,
            label: "room1".to_string(),
            desc: "".to_string(),
            exits: vec![],
        });

        let mob_player_0_id = r_mob(&mut game,"player0".to_string()).id;
        let mob_player_1_id = r_mob(&mut game,"player1".to_string()).id;

        game.player_connect("player0".to_string(), mob_player_0_id);
        game.player_connect( "player1".to_string(), mob_player_1_id);

        let gc = GameController::new(game);

        let map = gc.players_per_room();
        let result = map.get(&0);
        println!("{:?}", result);
        assert_eq!(result, Some(&vec![PlayerId(0), PlayerId(1)]));
    }
}
