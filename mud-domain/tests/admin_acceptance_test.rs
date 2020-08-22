use commons::{ConnectionId, DeltaTime};
use mud_domain::game::container::Container;
use mud_domain::game::{loader, Game, GameCfg};
use std::path::Path;

const delta_time: DeltaTime = DeltaTime(0.5);

#[test]
fn test_admin_menu() {
    // initialize with minimum config
    let mut game = setup();

    // connect and login
    let connection_id = ConnectionId(0);

    login(&mut game, connection_id);
    load_admin(&mut game, connection_id);

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

#[test]
fn test_admin_verify_with_valid_input() {
    // initialize with minimum config
    let mut game = setup();

    // connect and login
    let connection_id = ConnectionId(0);

    login(&mut game, connection_id);
    load_admin(&mut game, connection_id);

    game.handle_input(
        connection_id,
        r#"verify {
        "label": "not very well defined object"
    }"#,
    );
    game.tick(delta_time);

    let outputs = game.flush_outputs();
    assert_contains(outputs, "not very well defined object");
}

#[test]
fn test_admin_verify_with_invalid_input() {
    // initialize with minimum config
    let mut game = setup();

    // connect and login
    let connection_id = ConnectionId(0);

    login(&mut game, connection_id);
    load_admin(&mut game, connection_id);

    game.handle_input(
        connection_id,
        r#"verify {
        "label": "not very well defined object
    }"#,
    );
    game.tick(delta_time);

    let outputs = game.flush_outputs();
    assert_contains(outputs, "fail");
}

fn load_admin(game: &mut Game, connection_id: ConnectionId) {
    game.handle_input(connection_id, "admin");
    game.tick(delta_time);
    let outputs = game.flush_outputs();
    assert_contains(outputs, "admin");
}

fn login(game: &mut Game, connection_id: ConnectionId) {
    game.add_connection(connection_id);
    game.handle_input(connection_id, "player1");
    game.tick(delta_time);
    let _ = game.flush_outputs();
}

fn setup() -> Game {
    let mut container = Container::new();
    loader::Loader::load_folders(&mut container, &Path::new("../data/min")).unwrap();
    let mut game = Game::new(GameCfg::new(), container);
    game
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
