use super::domain::*;

/*
[You] [FireShield]
Hp: 12 Mana: 8
Defensive

[Enemy] [Recovery]
Hp: 10 Mana: 8
Casting Fireball (1)
 */
pub fn show_status(game: &mut Game) -> Vec<String> {
    let mut buffer = vec![];
    buffer.push(format!("Round {}", 0));
    buffer.push(format!("[You]   Hp: {} Mana: {} Actions: {}", 10, 10, 2));
    buffer.push(format!("[Enemy] Hp: {} Mana: {} Actions: {}", 10, 10, 2));
    buffer
}

/*
...@...
.......
.......
.......
...E...
 */
pub fn show_map(game: &Game) -> Vec<String> {
    let mut buffer = vec![];
    // draw the arena
    let arena = game.get_arena();
    for _lin in 0..arena.size[1] {
        let mut v = vec![];
        for _col in 0..arena.size[0] {
            v.push('.' as u8)
        }

        buffer.push(v);
    }

    // draw mobs
    for mob in game.get_mobs() {
        let lin = mob.pos[1] as usize;
        let col = mob.pos[0] as usize;

        let ch = if mob.is_player { '@' } else { 'O' };

        buffer[lin][col] = ch as u8;
    }

    buffer
        .into_iter()
        .map(|v| String::from_utf8(v).unwrap())
        .collect()
}

pub fn parse_input(game: &mut Game, line: &str) -> GResult<Command> {
    let trimmed = line.trim().split_ascii_whitespace().collect::<Vec<_>>();

    let command = trimmed
        .get(0)
        .ok_or(Error::Generic(format!("Invalid input {:?}", trimmed)))?;

    match *command {
        "exit" | "quit" => Ok(Command::Exit),
        "n" => Ok(Command::Move(Dir::N)),
        "e" => Ok(Command::Move(Dir::E)),
        "w" => Ok(Command::Move(Dir::W)),
        "s" => Ok(Command::Move(Dir::S)),
        _ => Err(Error::Generic(format!("Invalid input {:?}", trimmed))),
    }
}

pub fn show_events(game: &mut Game) -> Vec<String> {
    let mut buffer = vec![];
    buffer
}
