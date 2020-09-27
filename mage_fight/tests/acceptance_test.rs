use mage_fight::{console, domain::Game};

#[test]
fn acceptance_test() {
    let mut game = Game::new();
    game.start_game();

    // we should have 2 actions
    assert!(game.is_player_turn());

    let command = console::parse_input(&mut game, "s").unwrap();
    game.handle_player_command(command).unwrap();

    let command = console::parse_input(&mut game, "s").unwrap();
    game.handle_player_command(command).unwrap();

    // now is ai round, AI should have 2 actions
    assert!(!game.is_player_turn());
    game.handle_ai();

    assert!(!game.is_player_turn());
    game.handle_ai();

    // now we have a new round
    assert!(game.is_player_turn());

    let command = console::parse_input(&mut game, "c fb").unwrap();
    game.handle_player_command(command).unwrap();

    let lines = mage_fight::console::show_map(&game);
    for line in lines {
        println!("{}", line);
    }
}
