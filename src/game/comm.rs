use super::domain::*;

pub fn get_look_description(game: &Container, ctx: &PlayerCtx) -> String {
    let mut exits = vec![];
    for exit in &ctx.room.exits {
        let dir = &exit.0;
        exits.push(dir.to_string());
    }
    let exits = exits.join(", ");
    format!("{}\n\n{}\n\n[{}]\n", ctx.room.label, ctx.room.desc, exits).to_string()
}
