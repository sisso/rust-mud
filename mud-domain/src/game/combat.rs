use rand::Rng;

use super::comm;
use super::container::*;
use super::mob::*;
use super::Outputs;
use crate::errors::{AsResult, Error, Result};
use crate::game::mob;
use crate::errors::Error::InvalidStateFailure;
use logs::*;

pub fn tick_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
) -> Result<()> {
    let attacker = container.mobs.get(mob_id).ok_or(Error::NotFoundFailure)?;
    let defender = container.mobs.get(target_id);

    if let Some(defender) = defender {
        let attacker_room_id = container.locations.get(attacker.id).as_result()?;
        let defender_room_id = container.locations.get(defender.id).as_result()?;

        if attacker_room_id != defender_room_id {
            cancel_attack(container, outputs, mob_id, Some(&target_id));
            return Err(Error::InvalidArgumentFailure);
        }

        if attacker.is_read_to_attack(container.time.total) {
            execute_attack(container, outputs, mob_id, target_id)?;
        }

        match return_attack(container, outputs, target_id, mob_id) {
            Err(err) =>
                warn!("fail to execute return attack from {:?} to {:?}", mob_id, target_id),
            _ => {},
        };

        Ok(())
    } else {
        cancel_attack(container, outputs, mob_id, None);
        Ok(())
    }
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
// TODO: log errors, like can not get attribute?
fn execute_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
) -> Result<()> {
    let attacker_attributes = mob::get_attributes_with_bonus(container, mob_id)?;
    let attacker_room_id = container.locations.get(mob_id).as_result()?;
    let attacker_label = container.labels.get_label_f(mob_id);

    let defender_attributes = mob::get_attributes_with_bonus(container, mob_id)?;
    let defender_label = container.labels.get_label_f(target_id);

    let attack_result = roll_attack(
        attacker_attributes.attack,
        &attacker_attributes.damage,
        defender_attributes.defense,
        defender_attributes.rd
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
            mob.attributes.pv.current -= attack_result.damage_deliver as i32;
            dead = mob.attributes.pv.current < 0;
        }).unwrap();

        if dead {
            let defender_xp = container.mobs.get(target_id).unwrap().xp;

            container.mobs.update(mob_id, |mob| {
                mob.xp += defender_xp;
            })?;

            execute_attack_killed(container, outputs, mob_id, target_id, defender_xp)?;
        }
    }

    let total_time = container.time.total;

    container.mobs.update(mob_id, |mob| {
        mob.add_attack_calm_time(total_time);
    })?;

    Ok(())
}

fn execute_attack_killed(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
    xp: Xp,
) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let defender_label = container.labels.get_label_f(target_id);

    outputs.private(mob_id, comm::killed_by_player(defender_label, xp));
    outputs.broadcast(Some(mob_id), room_id, comm::killed(defender_label));

    mob::kill_mob(container, outputs, target_id)
}

fn roll_attack(attack: u32, damage: &Damage, defense: u32, rd: u32) -> AttackResult {
    let mut result = AttackResult::new(
        attack,
        defense,
        rd
    );

    result.attack_dice = roll_dice() + attack;
    result.defense_dice = roll_dice() + defense;
    result.success = result.attack_dice >= result.defense_dice;
    if result.success {
        result.damage_total = roll_damage(damage);
        result.damage_deliver = result.damage_total - rd;
    }

    result
}

fn roll_dice() -> u32 {
    let mut rng = rand::thread_rng();

    [0..2].iter().map(|_| rng.gen_range(1, 6 + 1)).sum()
}

fn roll_damage(damage: &Damage) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage.min, damage.max + 1)
}

/// mob_id is the mob that receive the attack
/// target_id is the mob that executed the attack
fn return_attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_id: MobId,
) -> Result<()> {
    let mob = container.mobs.get_mut(mob_id).as_result()?;

    if !mob.is_combat() {
        let location_id = container.locations.get(target_id).as_result()?;
        let mob_label = container.labels.get_label_f(mob_id);
        let aggressor_mob_label = container.labels.get_label_f(target_id);

        outputs.private(mob_id, comm::kill_return_attack_self(aggressor_mob_label));

        let msg = comm::kill_return_attack(mob_label, aggressor_mob_label);
        outputs.broadcast(Some(mob_id), location_id, msg);

        match mob.set_action_kill(target_id) {
            Err(err) =>
                warn!("{:?} fail to execute return attack to {:?}: {:?}", mob_id, target_id, err),
            Ok(_) =>
                info!("{:?} set attack to {:?}", mob_id, target_id),
        }
    }

    for follower_id in mob.followers.clone() {
        match return_attack(container, outputs, follower_id, target_id) {
            Err(err) =>
                warn!("{:?} fail to execute return attack from {:?} to {:?}: {:?}", follower_id, mob_id, target_id, err),
            _ => {},
        }
    }

    Ok(())
}
