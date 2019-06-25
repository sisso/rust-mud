mod command_handler;
mod controller;
mod domain;
mod player_input_handler;

use crate::server;
use crate::server::ConnectionId;

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

impl NewPlayerFactory for DefaultPlayerFactory {
    fn handle(&mut self, game: &mut Container, login: &String) -> PlayerId {
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

struct DefaultLoginView {

}

impl LoginView for DefaultLoginView {
    fn handle_welcome(&mut self, connection_id: &ConnectionId, outputs: &mut Vec<server::Output>) {
        let msg = controller::view_login::handle_welcome();

        outputs.push(server::Output {
            dest_connections_id: vec![connection_id.clone()],
            output: msg,
        });
    }

    fn handle(&mut self, game: &mut Container, server_outputs: &mut Vec<server::Output>, outputs: &mut Vec<Output>, connection_id: &ConnectionId, input: String, connection_state: &ConnectionState, player_factory: &mut NewPlayerFactory) -> Option<ConnectionState> {
        let result = view_login::handle(input);
        match result.login {
            Some(login) => {
                let player_id = player_factory.handle(game, &login).clone();

                // update local state
                let new_connection_state = ConnectionState::Logged {
                    connection_id: connection_id.clone(),
                    player_id: player_id,
                    login: login.clone(),
                };

                // handle output
                let look_output = command_handler::get_look_description(game, &game.get_player_context(&player_id));
                outputs.push(Output::private(player_id, format!("{}{}", result.msg, look_output)));

                Some(new_connection_state)
            },
            None => {
                server_outputs.push(server::Output {
                    dest_connections_id: vec![connection_id.clone()],
                    output: result.msg,
                });

                None
            },
        }
    }
}

pub fn run() {
    let mut game = Container::new();
    load_rooms(&mut game);
    let mut player_factory = DefaultPlayerFactory { room_id: 0 };
    let mut view_login = DefaultLoginView { };
    let mut player_input_handler = player_input_handler::DefaultPlayerInputHandler {};

    let mut game_controller = GameController::new();

    let mut server = server::Server::new();
    server.start();

    let mut pending_outputs: Vec<server::Output> = vec![];

    loop {
        let result = server.run(pending_outputs);

        let params = controller::GameControllerContext {
            game: &mut game,
            new_player_factory: &mut player_factory,
            view_login: &mut view_login,
            player_inputs_handler: &mut player_input_handler,
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs
        };

        pending_outputs = game_controller.handle(params);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
