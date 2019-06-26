mod actions;
mod comm;
mod controller;
mod domain;
mod players;
mod view_main;
mod view_login;

use crate::server;

use controller::*;
use domain::*;

fn load_rooms(game: &mut Container) {
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

struct DefaultPlayerFactory {
    room_id: u32,
}

pub fn run() {
    let mut game = Container::new();
    load_rooms(&mut game);
    let mut player_factory = DefaultPlayerFactory { room_id: 0 };
    let mut game_controller = GameController::new();

    let mut server = server::Server::new();
    server.start();

    let mut pending_outputs: Vec<server::Output> = vec![];

    loop {
        let result = server.run(pending_outputs);

        let params = controller::GameControllerContext {
            game: &mut game,
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs
        };

        pending_outputs = game_controller.handle(params);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
