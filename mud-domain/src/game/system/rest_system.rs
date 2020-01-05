use crate::game::system::SystemCtx;
use crate::errors::*;
use crate::game::comm;
use commons::{TotalTime, TimeTrigger};
use crate::game::mob::Mob;

pub fn run(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;

    for mob_id in ctx.container.mobs.list() {
        let mob = ctx.container.mobs.get_mut(mob_id).unwrap();
        
        if !mob.is_resting() {
            continue;
        }

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

    match TimeTrigger::check_trigger(
        mob.attributes.pv.heal_rate,
        mob.state.heal_calm_down,
        total,
    ) {
        Some(next) => {
            mob.state.heal_calm_down = next;
            mob.attributes.pv.current += 1;
            true
        }
        None => false,
    }
}

