mod game_controller;
mod game;
mod view_login;
mod view_mainloop;

use crate::server;
use game_controller::*;
use game::*;
use std::collections::{HashSet, HashMap};

fn load_rooms(game: &mut Game) {
    let room1 = Room {
        id: 0,
        label: "Main Room".to_string(),
        desc: "Main room where people born".to_string(),
        exits: vec![(Dir::S, 1)],
        // TODO: cam be simplified?
        tags: [RoomTag::INITIAL].iter().cloned().collect(),
    };

    let room2 = Room {
        id: 1,
        label: "Bar".to_string(),
        desc: "Where we relief our duties".to_string(),
        exits: vec![(Dir::N, 0)],
        tags: HashSet::new(),
    };

    game.add_room(room1);
    game.add_room(room2);
}

pub fn run() {
    let mut game = Game::new();
    load_rooms(&mut game);

    let mut game_controller = GameController::new(game);

    let mut server = server::Server::new();
    server.start();

    let mut pending_outputs: Vec<server::Output> = vec![];

    loop {
        let result = server.run(pending_outputs);
        let game_outputs= game_controller.handle(result.connects, result.disconnects, result.pending_inputs);
        pending_outputs = game_controller_output_to_server_output(game_outputs, game_controller.players_per_room());

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}

// TODO: move to game_controller???
fn game_controller_output_to_server_output(outputs: Vec<HandleOutput>, players_per_room: HashMap<u32, Vec<u32>>) -> Vec<server::Output> {
    let mut result = vec![];

    for mut i in outputs {
        let current_player_id = i.player_id;

        for player_msg in i.player_msg {
            println!("game_fantasy - sending to {:?}, '{}'", current_player_id, player_msg);

            let out = server::Output {
                dest_connections_id: vec![i.player_id],
                output: player_msg,
            };

            result.push(out);
        }

        for room_msg in i.room_msg {
            if let Some(players_in_room) = players_per_room.get(&i.room_id.expect("room msg without room id")) {
                let players_in_room = players_in_room.iter()
                    .flat_map(|player_id| {
                        if *player_id == current_player_id {
                            println!("game_fantasy - is same player {} {} ", player_id, current_player_id);
                            None
                        } else {
                            println!("game_fantasy - is other player {} {} ", player_id, current_player_id);
                            Some(*player_id)
                        }
                    }).collect();

                println!("game_fantasy - sending to {:?}, '{}'", players_in_room, room_msg);

                let out = server::Output {
                    dest_connections_id: players_in_room,
                    output: room_msg,
                };

                result.push(out);
            }
        }

    }

    result
}
