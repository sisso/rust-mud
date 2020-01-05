use crate::game::system::SystemCtx;
use crate::errors::*;
use crate::game::{comm, combat};
use crate::game::mob::MobCommand;
use logs::*;

pub fn run(ctx: &mut SystemCtx) {
    for mob_id in ctx.container.mobs.list() {
        let mob = ctx.container.mobs.get(mob_id).unwrap();

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
