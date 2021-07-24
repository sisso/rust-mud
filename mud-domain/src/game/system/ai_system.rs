use crate::errors::*;
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
        let result = match ai.command {
            AiCommand::Aggressive => run_aggressive(
                &mut container.mobs,
                &container.locations,
                &container.ownership,
                ai.id,
            ),
            _ => Ok(()),
        };

        match result {
            Err(e) => {
                warn!("fail to run ai for {:?}: {:?}", ai.id, e);
            }
            _ => {}
        }
    }
}

fn run_aggressive(
    mobs: &mut MobRepository,
    locations: &Locations,
    owners: &Ownerships,
    mob_id: ObjId,
) -> Result<()> {
    let mob = mobs.get(mob_id).as_result_str("mob not found")?;

    if !mob.command.is_idle() {
        return Ok(());
    }

    let location_id = locations.get(mob_id).as_result_str("mob has no location")?;

    for target_id in locations.list_at(location_id) {
        if combat::is_valid_attack_target(&mobs, &owners, mob_id, target_id) {
            let mob = mobs
                .get_mut(mob_id)
                .as_result_string(|| format!("mob {:?} not found", mob_id).into())?;
            match mob.set_action_attack(target_id) {
                Ok(()) => info!("{:?} aggressive attack {:?}", mob_id, target_id),
                Err(_e) => warn!("{:?} fail to attack {:?}", mob_id, target_id),
            }
            break;
        }
    }

    Ok(())
}
