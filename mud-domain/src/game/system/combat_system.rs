use crate::game::system::SystemCtx;
use crate::errors::*;
use crate::game::{comm, combat};
use crate::game::mob::MobCommand;
use logs::*;

// TODO: we should not have issues where the mob get deleted
pub fn run(ctx: &mut SystemCtx) {
    for mob_id in ctx.container.mobs.list() {
        // mob could have been deleted
        let mob = match ctx.container.mobs.get(mob_id) {
            Some(mob) => mob, 
            None => {
                warn!("mob_id {:?} not found", mob_id);
                continue
            },
        };

        let target_id = match mob.command {
            MobCommand::None => continue,
            MobCommand::Kill { target } => target,
        };

        combat::tick_attack(ctx.container, ctx.outputs, mob_id, target_id)
            .as_failure()
            .err()
            .iter()
            .for_each(|error| {
                warn!("{:?} fail to execute attack: {:?}", mob_id, error);
            });
    }
}
