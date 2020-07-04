use mage_fight::domain::{Command, Dir, Game};

#[test]
fn acceptance_test() {
    let mut game = Game::new();
    game.start_game();

    // we should have 2 actions
    assert!(game.is_player_turn());
    game.handle_player_command(Command::Move(Dir::S)).unwrap();

    assert!(game.is_player_turn());
    game.handle_player_command(Command::Move(Dir::S)).unwrap();

    // now is ai round, AI should have 2 actions
    assert!(!game.is_player_turn());
    game.handle_ai();
    assert!(!game.is_player_turn());
    game.handle_ai();

    // now we have a new round
    assert!(game.is_player_turn());
}
