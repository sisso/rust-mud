mod player_connection;
mod server;
mod view_login;
mod player;

use server::Server;

fn main() {
    let s = Server::new();
    s.run();
    println!("terminated");
}
