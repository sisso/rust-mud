mod server;
mod view_login;
mod view_mainloop;
mod game;

use server::Server;
use game::Game;
use game::Dir;
use game::Room;

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

fn main() {
    let mut game = Game::new();
    load_rooms(&mut game);

    let mut server = Server::new();
    server.start();
    loop {
        server.run();
        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
    println!("terminated");
}
