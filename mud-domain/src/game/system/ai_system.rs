use crate::game::ai::AiCommand;
use crate::game::combat;
use crate::game::container::Container;
use crate::game::location::Locations;
use crate::game::mob::{MobCommand, MobRepository};
use crate::game::ownership::Ownerships;
use commons::unwrap_or_return;
use commons::ObjId;
use logs::*;

pub fn run(container: &mut Container) {
    for ai in container.ai.list() {
        match ai.command {
            AiCommand::Aggressive => run_aggressive(
                &mut container.mobs,
                &container.locations,
                &container.ownership,
                ai.id,
            ),
            _ => {}
        }
    }
}

fn run_aggressive(
    mobs: &mut MobRepository,
    locations: &Locations,
    owners: &Ownerships,
    mob_id: ObjId,
) {
    let mob = unwrap_or_return!(mobs.get(mob_id));

    if !mob.command.is_idle() {
        return;
    }

    let location_id = unwrap_or_return!(locations.get(mob_id));

    for target_id in locations.list_at(location_id) {
        if combat::is_valid_attack_target(&mobs, &owners, mob_id, target_id) {
            let mob = unwrap_or_return!(mobs.get_mut(mob_id));
            match mob.set_action_attack(target_id) {
                Ok(()) => info!("{:?} aggressive attack {:?}", mob_id, target_id),
                Err(_e) => warn!("{:?} fail to attack {:?}", mob_id, target_id),
            }
            break;
        }
    }
}
