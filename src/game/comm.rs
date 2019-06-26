use super::domain::*;

pub fn look_description(game: &Container, ctx: &PlayerCtx) -> String {
    let mut exits = vec![];
    for exit in &ctx.room.exits {
        let dir = &exit.0;
        exits.push(dir.to_string());
    }
    let exits = exits.join(", ");
    format!("{}\n\n{}\n\n[{}]\n", ctx.room.label, ctx.room.desc, exits).to_string()
}

pub fn unknown_input(input: String) -> String {
    format!("unknown command '{}'", input)
}

pub fn say_you_say(msg: &String) -> String {
    format!("you say '{}'\n", msg)
}

pub fn say_someone_said(actor: &String, msg: &String) -> String {
    format!("{} says '{}'\n", actor, msg)
}

pub fn move_you_move(dir: &Dir) -> String {
    format!("you move to {}!", dir)
}

pub fn move_come(who: &String, dir: &Dir) -> String {
    format!("{} comes from {}.\n", who, dir)
}

pub fn move_goes(who: &String, dir: &Dir) -> String {
    format!("{} goes to {}.\n", who, dir)
}

pub fn move_not_possible(dir: &Dir) -> String {
    format!("not possible to move to {}!\n", dir)
}
