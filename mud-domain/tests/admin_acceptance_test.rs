use commons::{ConnectionId, DeltaTime};
use mud_domain::game::container::Container;
use mud_domain::game::{loader, Game, GameCfg};
use std::path::Path;

#[test]
fn test_admin_menu() {
    // initialize with minimum config
    let mut container = Container::new();
    loader::Loader::load_folders(&mut container, &Path::new("../data/min")).unwrap();
    let mut game = Game::new(GameCfg::new(), container);

    // connect and login
    let delta_time = DeltaTime(0.5);
    let connection_id = ConnectionId(0);
    game.add_connection(connection_id);
    game.handle_input(connection_id, "player1");
    game.tick(delta_time);
    let _ = game.flush_outputs();

    game.handle_input(connection_id, "admin");
    game.tick(delta_time);
    let outputs = game.flush_outputs();
    assert_contains(outputs, "admin");

    game.handle_input(connection_id, "list");
    game.tick(delta_time);
    let outputs = game.flush_outputs();
    assert_contains(outputs, "player1");

    game.handle_input(connection_id, "get 1");
    game.tick(delta_time);
    let outputs = game.flush_outputs();
    assert_contains(outputs, "void");

    game.handle_input(connection_id, "ls p mob");
    game.tick(delta_time);
    let outputs = game.flush_outputs();
    assert_contains(outputs, "God");
}

fn assert_contains(outputs: Vec<(ConnectionId, String)>, s: &str) {
    let outputs: Vec<String> = outputs.into_iter().map(|i| i.1).collect();

    for msg in &outputs {
        if msg.to_ascii_lowercase().contains(&s.to_ascii_lowercase()) {
            return;
        }
    }

    panic!("could not found [{}] in outputs: {:?}", s, outputs);
}
