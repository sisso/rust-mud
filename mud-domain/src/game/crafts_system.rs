use crate::game::container::Ctx;
use crate::game::crafts::CraftCommand;
use crate::utils::geometry;
use crate::game::{comm};

pub fn tick(ctx: &mut Ctx) {
    let mut commands_complete = vec![];

    for craft in ctx.container.crafts.list_all() {
        match craft.command {
            CraftCommand::Idle => {},
            CraftCommand::MoveTo { target_id }=> {
                let target_pos = ctx.container.pos.get_pos(target_id);
                let self_pos = ctx.container.pos.get_pos(craft.id);

                match (self_pos, target_pos) {
                    (Some(self_pos), Some(target_pos)) => {
                        let max_distance = craft.attributes.speed * ctx.container.time.delta.as_f32();
                        let (new_pos, done) = geometry::move_towards(self_pos, target_pos, max_distance);
                        let _ = ctx.container.pos.set_pos(craft.id, new_pos);
                        if done {
                            commands_complete.push((craft.id, true));
                        }
                    },
                    _ => {
                        commands_complete.push((craft.id, false));
                    }
                }
            },
        }
    }

    for (craft_id, success) in commands_complete {
        ctx.container.crafts.set_command(craft_id, CraftCommand::Idle).unwrap();

        let msg =
            if success { comm::space_command_complete() }
            else { comm::space_command_failed() };
        ctx.outputs.zone_all(craft_id, msg);
    }
}
