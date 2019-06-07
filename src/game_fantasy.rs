mod game_controller;
mod game;
mod view_login;
mod view_mainloop;

use crate::server;
use game_controller::*;
use game::*;

fn load_rooms(game: &mut Game) {
    let room1 = Room {
        id: 0,
        label: "Main Room".to_string(),
        desc: "Main room where people born".to_string(),
        exits: vec![(Dir::S, 1)],
    };

    let room2 = Room {
        id: 1,
        label: "Bar".to_string(),
        desc: "Where we relief our duties".to_string(),
        exits: vec![(Dir::N, 0)],
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
        pending_outputs= game_controller.handle(result.connects, result.disconnects, result.pending_inputs);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
