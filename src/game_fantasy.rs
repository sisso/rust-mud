mod game_controller;
mod game;
mod view_login;
mod view_mainloop;

use crate::server;
use game_controller::*;
use game::*;
use std::collections::HashSet;
use crate::server::Output;
use core::borrow::Borrow;

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
        pending_outputs = game_controller_output_to_server_output(game_outputs);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}

fn game_controller_output_to_server_output(outputs: Vec<HandleOutput>) -> Vec<server::Output> {
    let mut result = vec![];

    for mut i in outputs {
        if !i.room_id.is_empty() || !i.room_msg.is_empty() || i.player_msg.len() > 1 {
            panic!("not supported")
        }

        let out = server::Output {
            dest_connections_id: vec![i.player_id],
            output: i.player_msg.pop().expect("not supported"),
        };

        result.push(out);
    }

    result
}
