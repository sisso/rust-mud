use rand::Rng;

use super::body;
use super::comm;
use super::domain::*;
use super::container::*;
use super::controller::Outputs;
use super::mob::*;

pub fn tick_attack(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: &MobId, target_mob_id: &MobId) {
    let attacker = container.mobs.get(&mob_id);
    let defender = container.mobs.find(&target_mob_id);

    if let Some(defender) = defender {
        // TODO: how send references?
        if attacker.room_id != defender.room_id {
            cancel_attack(container, outputs, &mob_id, Some(target_mob_id));
            return;
        }

        if attacker.is_read_to_attack(&time.total) {
            execute_attack(time, container, outputs, &mob_id, &target_mob_id);
        }

        check_return_attack(time, container, outputs, target_mob_id, mob_id);
    } else {
        cancel_attack(container, outputs, &mob_id, None);
    }
}

fn cancel_attack(container: &mut Container, outputs: &mut dyn Outputs, mob_id: &MobId, target: Option<&MobId>) {
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
    let mut mob = container.mobs.get(&mob_id).clone();
    mob.command = MobCommand::Idle;
    container.mobs.update(mob);
}

fn execute_attack(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: &MobId, target: &MobId) {
    let player_id = container.players.find_player_id_from_avatar_mob_id(mob_id);

    let attacker = container.mobs.get(&mob_id);
    let defender = container.mobs.get(&target);

    let attack_result = roll_attack(&attacker.attributes.attack, &attacker.attributes.damage, &defender.attributes.defense);
    let room_attack_msg = comm::kill_mob_execute_attack(attacker, defender, &attack_result);

    if let Some(player_id) = player_id {
        let player_attack_msg = comm::kill_player_execute_attack(&defender, &attack_result);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    if attack_result.success {
        // TODO: remove get/update/get
        // deduct pv
        let mut defender = container.mobs.get(&target).clone();
        defender.attributes.pv.current -= attack_result.damage as i32;
        container.mobs.update(defender);

        let defender = container.mobs.get(&target);
        if defender.attributes.pv.current < 0 {
            kill_mob(time, container, outputs, mob_id, target);
        }
    }

    let mut attacker = container.mobs.get(&mob_id).clone();
    attacker.add_attack_calm_time(&time.total);
    container.mobs.update(attacker);
}

// TODO: create body
fn kill_mob(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, attacker_id: &MobId, target_id: &MobId) {
    let attacker_player_id = container.players.find_player_id_from_avatar_mob_id(attacker_id);
    let attacker = container.mobs.get(&attacker_id);
    let defender = container.mobs.get(&target_id);

    let room_attack_msg = comm::killed(&defender);

    if let Some(player_id) = attacker_player_id {
        let player_attack_msg = comm::killed_by_player(&defender);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    body::create_body(time, container, outputs, target_id);
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


fn check_return_attack(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: &MobId, aggressor_mob_id: &MobId) {
    match container.mobs.find(mob_id) {
        Some(mob) if mob.command.is_idle() => {
            let aggressor_mob = container.mobs.get(aggressor_mob_id);
            let msg = comm::kill_return_attack(&mob.label, &aggressor_mob.label);
            outputs.room_all(mob.room_id, msg);

            container.mobs.set_mob_kill_target(mob_id, aggressor_mob_id);
        },
        _ => {}
    }
}
