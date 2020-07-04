use mage_fight::console::*;
use mage_fight::domain::*;

use std::io::stdin;

fn main() {
    let mut game = Game::new();
    game.start_game();

    loop {
        let buffer = show_status(&mut game);
        for line in buffer {
            println!("{}", line);
        }

        let buffer = show_map(&mut game);
        for line in buffer {
            println!("{}", line);
        }

        if game.is_player_turn() {
            let mut line = String::new();
            println!("command:");

            stdin()
                .read_line(&mut line)
                .expect("fail to read command line");

            match handle_input(&mut game, line.as_str()) {
                Ok(command) => match game.handle_player_command(command) {
                    Ok(()) => break,
                    Err(Error::Generic(msg)) => {
                        println!("{}", msg);
                    }
                    other => panic!("unexpected error {:?}", other),
                },
                Err(Error::Generic(msg)) => {
                    println!("{}", msg);
                }
                other => panic!("unexpected error {:?}", other),
            }
        } else {
            game.handle_ai();
        }

        let buffer = show_events(&mut game);
        for line in buffer {
            println!("{}", line);
        }
    }
}
