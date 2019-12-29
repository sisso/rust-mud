use rand::Rng;

use super::comm;
use super::container::*;
use super::mob::*;
use super::Outputs;
use crate::errors::{AsResult, Error, Result};
use crate::game::mob;

pub fn tick_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_mob_id: MobId,
) -> Result<()> {
    let attacker = container.mobs.get(mob_id).ok_or(Error::NotFound)?;
    let defender = container.mobs.get(target_mob_id);

    if let Some(defender) = defender {
        let attacker_room_id = container.locations.get(attacker.id).as_result()?;
        let defender_room_id = container.locations.get(defender.id).as_result()?;

        if attacker_room_id != defender_room_id {
            cancel_attack(container, outputs, mob_id, Some(&target_mob_id));
            return Err(Error::IllegalArgument);
        }

        if attacker.is_read_to_attack(container.time.total) {
            execute_attack(container, outputs, mob_id, target_mob_id);
        }

        check_return_attack(container, outputs, target_mob_id, mob_id);
    } else {
        cancel_attack(container, outputs, mob_id, None);
    }

    Ok(())
}

fn cancel_attack(
    container: &mut Container,
    _outputs: &mut dyn Outputs,
    mob_id: MobId,
    _target: Option<&MobId>,
) {
    //    let attacker = container.get_mob(&mob_id.0);

    //    let msg_others = comm::kill_cancel(attacker, defender);
    //
    //    if attacker.is_avatar {
    //        let player = container.find_player_from_avatar_mob_id(&MobId(attacker.id)).unwrap();
    //        let msg_player = comm::kill_player_cancel(defender);
    //        outputs.private(player.id.clone(), msg_player);
    //        outputs.room(player.id.clone(), attacker.room_id,msg_others);
    //    } else {
    //        outputs.room_all( attacker.room_id, msg_others);
    //    }

    //    let mut mob = attacker.clone();
    container.mobs.cancel_attack(mob_id);
}

// TODO: fix multiples get from get two mutable
fn execute_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
) -> Result<()> {
    let attacker = container.mobs.get(mob_id).as_result()?;
    let attacker_room_id = container.locations.get(mob_id).as_result()?;
    let attacker_label = container.labels.get_label_f(mob_id);

    let defender = container.mobs.get(target_id).as_result()?;
    let defender_label = container.labels.get_label_f(target_id);

    let attack_result = roll_attack(
        &attacker.attributes.attack,
        &attacker.attributes.damage,
        &defender.attributes.defense,
    );
    let room_attack_msg =
        comm::kill_mob_execute_attack(attacker_label, defender_label, &attack_result);

    let player_attack_msg = comm::kill_player_execute_attack(defender_label, &attack_result);
    outputs.private(mob_id, player_attack_msg);
    outputs.broadcast(Some(mob_id), attacker_room_id, room_attack_msg);

    if attack_result.success {
        // deduct pv
        let mut dead = false;
        container.mobs.update(target_id, |mob| {
            mob.attributes.pv.current -= attack_result.damage as i32;
            dead = mob.attributes.pv.current < 0;
        })?;

        if dead {
            execute_attack_killed(container, outputs, mob_id, target_id);
        }
    }

    let total_time = container.time.total;

    container.mobs.update(mob_id, |mob| {
        mob.add_attack_calm_time(total_time);
    });

    Ok(())
}

fn execute_attack_killed(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let defender_label = container.labels.get_label_f(target_id);

    outputs.private(mob_id, comm::killed_by_player(defender_label));
    outputs.broadcast(Some(mob_id), room_id, comm::killed(defender_label));

    mob::kill_mob(container, outputs, target_id)
}

fn roll_attack(attack: &u32, damage: &Damage, defense: &u32) -> AttackResult {
    let attack_dice = roll_dice() + *attack;
    let defense_dice = roll_dice() + *defense;

    if attack_dice < defense_dice {
        return AttackResult {
            success: false,
            damage: 0,
            attack_dice,
            defense_dice,
        };
    }

    let damage = roll_damage(damage);

    AttackResult {
        success: true,
        damage,
        attack_dice,
        defense_dice,
    }
}

fn roll_dice() -> u32 {
    let mut rng = rand::thread_rng();

    [0..2].iter().map(|_| rng.gen_range(1, 6 + 1)).sum()
}

fn roll_damage(damage: &Damage) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage.min, damage.max + 1)
}

fn check_return_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    aggressor_mob_id: MobId,
) -> Result<()> {
    match container.mobs.get(mob_id) {
        Some(mob) if mob.command.is_idle() => {
            let _aggressor_mob = container.mobs.get(aggressor_mob_id).as_result()?;
            let room_id = container.locations.get(aggressor_mob_id).as_result()?;
            let mob_label = container.labels.get_label_f(mob_id);
            let aggressor_mob_label = container.labels.get_label_f(aggressor_mob_id);

            let msg = comm::kill_return_attack(mob_label, aggressor_mob_label);
            outputs.broadcast(None, room_id, msg);

            container
                .mobs
                .set_mob_attack_target(mob_id, aggressor_mob_id);
        }
        _ => {}
    }

    Ok(())
}
