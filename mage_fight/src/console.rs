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
pub fn show_map(game: &mut Game) -> Vec<String> {
    let mut buffer = vec![];
    buffer.push("....Y....".to_string());
    buffer.push(".........".to_string());
    buffer.push(".........".to_string());
    buffer.push(".........".to_string());
    buffer.push(".........".to_string());
    buffer.push("....E....".to_string());
    buffer
}

pub fn handle_input(game: &mut Game, line: &str) -> Result<Command> {
    let trimmed = line.trim();
    Err(Error::Generic(format!("Invalid input {:?}", trimmed)))
}

pub fn show_events(game: &mut Game) -> Vec<String> {
    let mut buffer = vec![];
    buffer
}
