mod game_controller;
mod game;
mod view_login;
mod view_mainloop;

use crate::server::Server;

use game_controller::*;
use game::*;

fn load_rooms(game: &mut Game) {
    let room1 = Room {
        id: 0,
        name: "Main Room".to_string(),
        exits: vec![(Dir::S, 1)]
    };

    let room2 = Room {
        id: 1,
        name: "Bar".to_string(),
        exits: vec![(Dir::N, 0)]
    };

    game.add_room(room1);
    game.add_room(room2);
}

pub fn run() {
    let mut game = Game::new();
    load_rooms(&mut game);

    let mut game_controller = GameController::new(game);

    let mut server = Server::new();
    server.start();

    let mut pending_outputs: Vec<(u32, String)> = vec![];

    loop {
        let result = server.run(pending_outputs);
        pending_outputs = game_controller.handle(result.connects, result.disconnects, result.pending_inputs);
        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
    println!("terminated");
}
