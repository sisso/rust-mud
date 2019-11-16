use crate::game::container::Ctx;
use crate::game::crafts::CraftCommand;
use crate::utils::geometry;
use logs::*;

pub fn tick(ctx: &mut Ctx) {
    let mut changes = vec![];

    for craft in ctx.container.crafts.list_all() {
        match craft.command {
            CraftCommand::Idle => {},
            CraftCommand::MoveTo { target_id }=> {
                let target_pos = ctx.container.pos.get_pos(target_id);
                let self_pos = ctx.container.pos.get_pos(craft.id);

                match (self_pos, target_pos) {
                    (Ok(self_pos), Ok(target_pos)) => {
                        let max_distance = craft.attributes.speed * ctx.container.time.delta.as_f32();
                        let (new_pos, done) = geometry::move_towards(self_pos, target_pos, max_distance);
                        let _ = ctx.container.pos.set_pos(craft.id, new_pos);
                        if done {
                            changes.push(craft.id);
                        }
                    },
                    _ => {
                        changes.push(craft.id);
                    }
                }
            },
        }
    }
}
