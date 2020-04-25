use crate::errors::*;
use crate::game::mob::MobCommand;
use crate::game::system::SystemCtx;
use crate::game::triggers::EventKind;
use crate::game::{combat, comm};
use commons::unwrap_or_continue;
use logs::*;
use std::sync::atomic::Ordering::AcqRel;

// TODO: we should not have issues where the mob get deleted
pub fn run(ctx: &mut SystemCtx) {
    run_combat(ctx);
}

fn run_combat(ctx: &mut SystemCtx) {
    let mut attacks = vec![];
    let mut aggressive = vec![];

    for mob in ctx.container.mobs.list() {
        match mob.command {
            MobCommand::None if mob.aggressive && !mob.is_combat() => aggressive.push(mob.id),
            MobCommand::None => {}
            MobCommand::Kill { target_id } => attacks.push((mob.id, target_id)),
        };
    }

    // check aggressive for target
    for mob_id in aggressive {
        let location_id = unwrap_or_continue!(ctx.container.locations.get(mob_id));

        for target_id in ctx.container.locations.list_at(location_id) {
            if target_id == mob_id {
                continue;
            }

            if ctx.container.ownership.same_owner(mob_id, target_id) {
                continue;
            }

            let mob = unwrap_or_continue!(ctx.container.mobs.get_mut(mob_id));

            match mob.set_action_attack(target_id) {
                Ok(()) => info!("{:?} aggressive attack {:?}", mob_id, target_id),
                Err(e) => warn!("{:?} fail to attack {:?}", mob_id, target_id),
            }

            break;
        }
    }

    // execute attacks
    for (mob_id, target_id) in &attacks {
        match combat::tick_attack(ctx.container, ctx.outputs, *mob_id, *target_id) {
            Err(err) => warn!("{:?} fail to execute attack: {:?}", mob_id, err),
            _ => {}
        };
    }
}
