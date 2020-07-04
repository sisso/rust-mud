use commons::{ConnectionId, DeltaTime};
use mud_domain::game::container::Container;
use mud_domain::game::loader::dto::LoaderData;
use mud_domain::game::{loader, Game, GameCfg};
use std::path::Path;

#[test]
fn test_serialization_and_reload_in_game() {
    let snapshot_1 = {
        let mut container = Container::new();
        loader::Loader::load_folders(&mut container, &Path::new("../data/fantasy")).unwrap();

        let mut game = Game::new(GameCfg::new(), container);
        game.tick(DeltaTime(0.5));

        let connection_id = ConnectionId(0);
        game.add_connection(connection_id);
        game.handle_input(connection_id, "player1");

        for _ in 0..100 {
            game.tick(DeltaTime(0.5));
        }

        loader::Loader::create_snapshot(&game.container).unwrap()
    };

    let snapshot_2 = {
        let mut container = Container::new();
        loader::Loader::load_data(&mut container, snapshot_1.clone());
        loader::Loader::create_snapshot(&container).unwrap()
    };

    assert_snapshots(&snapshot_1, &snapshot_2);
}

fn assert_snapshots(snapshot_1: &LoaderData, snapshot_2: &LoaderData) {
    let mut list_1: Vec<_> = snapshot_1.objects.iter().collect();
    list_1.sort_by_key(|(key, _)| key.0);

    let mut list_2: Vec<_> = snapshot_2.objects.iter().collect();
    list_2.sort_by_key(|(key, _)| key.0);

    assert_json_eq(&list_2, &list_1);
}

fn assert_json_eq<T: serde::ser::Serialize>(value: &T, expected: &T) {
    let json_value = serde_json::to_string_pretty(value).expect("value can not be serialized");
    let json_expected =
        serde_json::to_string_pretty(expected).expect("expected can not be serialized");

    std::fs::write("/tmp/01.json", json_value.clone()).unwrap();

    assert_eq!(json_value, json_expected);
}
