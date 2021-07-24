use crate::errors::*;
use crate::game;
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
    let mut to_drop = vec![];

    for ai in container.ai.list() {
        let result = match ai.command {
            AiCommand::Aggressive => run_aggressive(
                &mut container.mobs,
                &container.locations,
                &container.ownership,
                ai.id,
            ),
            AiCommand::Extract { .. } => {
                // drop any item
                let items: Vec<ObjId> = container.locations.list_at(ai.id).collect();
                to_drop.push((ai.id, items));
                Ok(())
            }
            _ => Ok(()),
        };

        match result {
            Err(e) => {
                warn!("fail to run ai for {:?}: {:?}", ai.id, e);
            }
            _ => {}
        }
    }

    for (obj_id, items) in to_drop {
        for item_id in items {
            match game::actions_items::do_drop(container, obj_id, item_id) {
                Err(e) => warn!("{:?} fail to drop {:?}: {:?}", obj_id, item_id, e),
                _ => {}
            }
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

#[cfg(test)]
mod test {
    use crate::game::loader::Loader;
    use crate::game::triggers::EventKind;
    use crate::game::{self, builder};
    use commons::{ObjId, TotalTime};

    #[test]
    fn ai_extracting_should_drop_objects_at_ground_when_inventory_is_full() {
        let mut scenery = crate::game::test::scenery();
        let lc = Loader::load_hocon_files(
            &mut scenery.container,
            &vec!["../data/tests/scenery_mining_bot.conf"],
        )
        .unwrap();

        let mining_bot_id = lc.get(4);
        let location_id = lc.get(2);
        let extractable_id = lc.get(3);

        // move miner to same right location
        scenery.container.locations.set(mining_bot_id, location_id);

        // set it to miner
        crate::game::actions_command::set_command_extract(
            &mut scenery.container,
            mining_bot_id,
            location_id,
            extractable_id,
        )
        .unwrap();

        let mut full_inv = false;
        let mut on_ground = false;

        for _ in 0..100 {
            scenery.tick(1.0);

            let inv = scenery.container.inventories.get(mining_bot_id).unwrap();
            let miner_children: Vec<_> =
                scenery.container.locations.list_at(mining_bot_id).collect();

            full_inv = inv.current_weight.unwrap_or(0.0) > 0.0 && !miner_children.is_empty();
            // the mine bot, the mine resource, the mine ore dropped by the bot
            on_ground = scenery.container.locations.list_at(location_id).count() > 2;
        }

        assert!(full_inv);
        assert!(on_ground);
    }
}
