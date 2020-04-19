use crate::errors::*;
use crate::game::comm;
use crate::game::mob::Mob;
use crate::game::system::SystemCtx;
use commons::{TimeTrigger, TotalTime};

pub fn run(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;

    for mob in ctx.container.mobs.list_mut() {
        if !mob.is_resting() {
            continue;
        }

        let mob_id = mob.id;

        if update_resting(mob, total_time) {
            if mob.attributes.pv.is_damaged() {
                let msg = comm::rest_healing(mob.attributes.pv.current);
                ctx.outputs.private(mob_id, msg);
            } else {
                ctx.outputs.private(mob_id, comm::rest_healed());
            }
        }
    }
}

fn update_resting(mob: &mut Mob, total: TotalTime) -> bool {
    if !mob.attributes.pv.is_damaged() {
        return false;
    }

    match TimeTrigger::check_trigger(mob.attributes.pv.heal_rate, mob.state.heal_calm_down, total) {
        Some(next) => {
            mob.state.heal_calm_down = next;
            mob.attributes.pv.current += 1;
            true
        }
        None => false,
    }
}
