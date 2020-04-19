use crate::errors::*;
use crate::game::mob::MobCommand;
use crate::game::system::SystemCtx;
use crate::game::{combat, comm};
use commons::unwrap_or_continue;
use logs::*;

// TODO: we should not have issues where the mob get deleted
pub fn run(ctx: &mut SystemCtx) {
    let mut attacks = vec![];

    for mob in ctx.container.mobs.list() {
        let target_id = match mob.command {
            MobCommand::None => continue,
            MobCommand::Kill { target_id } => target_id,
        };

        attacks.push((mob.id, target_id));
    }

    // execute attacks
    for (mob_id, target_id) in &attacks {
        match combat::tick_attack(ctx.container, ctx.outputs, *mob_id, *target_id) {
            Err(err) => warn!("{:?} fail to execute attack: {:?}", mob_id, err),
            _ => {}
        };
    }
}
