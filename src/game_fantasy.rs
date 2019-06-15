mod game;
mod game_controller;
mod command_handler;
mod view_login;
mod view_mainloop;

use crate::server;
use game_controller::*;
use game::*;
use std::panic::resume_unwind;

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

struct DefaultPlayerFactory {
    room_id: u32,
}

impl NewPlayerFactory for DefaultPlayerFactory {
    fn handle(&mut self, game: &mut Game, login: &String) -> PlayerId {
        // add player avatar
        let mob_id = game.next_mob_id();
        
        let mob = Mob {
            id: mob_id,
            label: login.clone(),
            room_id: self.room_id,
            is_avatar: true
        };

        game.add_mob(mob);

        // add player to game
        let player = game.player_connect(login.clone(), mob_id);
        player.id
    }
}

pub fn run() {
    let mut game = Game::new();
    load_rooms(&mut game);
    let mut player_factory = DefaultPlayerFactory { room_id: 0 };

    let mut game_controller = GameController::new();

    let mut server = server::Server::new();
    server.start();

    let mut pending_outputs: Vec<server::Output> = vec![];

    loop {
        let result = server.run(pending_outputs);

        let params = game_controller::GameControllerContext {
            game: &mut game,
            new_player_factory: &mut player_factory,
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs
        };

        pending_outputs = game_controller.handle(params);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
