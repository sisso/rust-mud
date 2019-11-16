use rand::Rng;

use super::comm;
use super::container::*;
use super::Outputs;
use super::mob::*;
use crate::game::mob;

pub fn tick_attack(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, target_mob_id: MobId) -> Result<(), ()> {
    let attacker = container.mobs.get(mob_id)?;
    let defender = container.mobs.get(target_mob_id);

    if let Ok(defender) = defender {
        let attacker_room_id = container.locations.get(attacker.id)?;
        let defender_room_id = container.locations.get(defender.id)?;

        if attacker_room_id != defender_room_id {
            cancel_attack(container, outputs, mob_id, Some(&target_mob_id));
            return Err(());
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

fn cancel_attack(container: &mut Container, _outputs: &mut dyn Outputs, mob_id: MobId, _target: Option<&MobId>) {
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
fn execute_attack(container: &mut Container, outputs: &mut dyn Outputs, attacker_id: MobId, target_id: MobId) -> Result<(),()> {
    let player_id = container.players.find_from_mob(attacker_id);

    let attacker = container.mobs.get(attacker_id)?;
    let attacker_room_id = container.locations.get(attacker_id)?;
    let attacker_label = container.labels.get_label_f(attacker_id);

    let defender = container.mobs.get(target_id)?;
    let defender_label = container.labels.get_label_f(target_id);

    let attack_result = roll_attack(&attacker.attributes.attack, &attacker.attributes.damage, &defender.attributes.defense);
    let room_attack_msg = comm::kill_mob_execute_attack(attacker_label, defender_label, &attack_result);

    if let Ok(player_id) = player_id {
        let player_attack_msg = comm::kill_player_execute_attack(defender_label, &attack_result);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker_room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker_room_id, room_attack_msg);
    }

    if attack_result.success {
        // TODO: remove get/update/get
        // deduct pv
        let mut defender = container.mobs.get(target_id)?.clone();
        defender.attributes.pv.current -= attack_result.damage as i32;
        container.mobs.update(defender);

        let defender = container.mobs.get(target_id)?;
        if defender.attributes.pv.current < 0 {
            execute_attack_killed(container, outputs, attacker_id, target_id);
        }
    }

    let mut attacker = container.mobs.get(attacker_id)?.clone();
    attacker.add_attack_calm_time(container.time.total);
    container.mobs.update(attacker);

    Ok(())
}

fn execute_attack_killed(container: &mut Container, outputs: &mut dyn Outputs, attacker_id: MobId, target_id: MobId) -> Result<(),()> {
    let attacker_player_id = container.players.find_from_mob(attacker_id);
    let _attacker = container.mobs.get(attacker_id)?;
    let _attacker_label = container.labels.get_label_f(attacker_id);
    let attacker_room_id = container.locations.get(attacker_id)?;
    let _defender = container.mobs.get(target_id)?;
    let defender_label = container.labels.get_label_f(target_id);

    let room_attack_msg = comm::killed(defender_label);

    if let Ok(player_id) = attacker_player_id {
        let player_attack_msg = comm::killed_by_player(defender_label);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker_room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker_room_id, room_attack_msg);
    }

    mob::kill_mob(container, outputs, target_id);
    Ok(())
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

    [0..2].iter()
        .map(|_| rng.gen_range(1, 6 + 1))
        .sum()
}

fn roll_damage(damage: &Damage) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage.min, damage.max + 1)
}


fn check_return_attack(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, aggressor_mob_id: MobId) -> Result<(),()> {
    match container.mobs.get(mob_id) {
        Ok(mob) if mob.command.is_idle() => {
            let _aggressor_mob = container.mobs.get(aggressor_mob_id)?;
            let room_id = container.locations.get(aggressor_mob_id)?;
            let mob_label = container.labels.get_label_f(mob_id);
            let aggressor_mob_label = container.labels.get_label_f(aggressor_mob_id);

            let msg = comm::kill_return_attack(mob_label, aggressor_mob_label);
            outputs.room_all(room_id, msg);

            container.mobs.set_mob_attack_target(mob_id, aggressor_mob_id);
        },
        _ => {}
    }

    Ok(())
}
