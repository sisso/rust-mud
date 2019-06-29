use super::domain::*;
use super::mob::*;

pub fn look_description(container: &Container, ctx: &PlayerCtx) -> String {
    let mut exits = vec![];
    for exit in &ctx.room.exits {
        let dir = &exit.0;
        exits.push(dir.to_string());
    }
    let exits = exits.join(", ");
    let mobs = container.find_mobs_at(&ctx.avatar.room_id);
    let mobs =
        if mobs.is_empty() {
            "".to_string()
        } else {
            let labels: Vec<String> =
                mobs.iter()
                    .filter(|i| i.id != ctx.avatar.id)
                    .map(|i| {
                        format!("- {} is here", i.label)
                    }).collect();

            labels.join("\n")
        };

    format!("{}\n\n{}\n\n[{}]\n\n{}", ctx.room.label, ctx.room.desc, exits, mobs).to_string()
}

pub fn unknown_input(input: String) -> String {
    format!("unknown command '{}'", input)
}

pub fn say_you_say(msg: &String) -> String {
    format!("you say '{}'\n", msg)
}

pub fn say_someone_said(actor: &String, msg: &String) -> String {
    format!("{} says '{}'\n", actor, msg)
}

pub fn move_you_move(dir: &Dir) -> String {
    format!("you move to {}!", dir)
}

pub fn move_come(who: &String, dir: &Dir) -> String {
    format!("{} comes from {}.\n", who, dir)
}

pub fn move_goes(who: &String, dir: &Dir) -> String {
    format!("{} goes to {}.\n", who, dir)
}

pub fn move_not_possible(dir: &Dir) -> String {
    format!("not possible to move to {}!\n", dir)
}

pub fn spawn_mob(mob: &Mob) -> String {
    format!("a {} appears here from no where", mob.label)
}

pub fn uptime(time: &Seconds) -> String {
    format!("now it is {} seconds after start", time.0)
}

pub fn kill_target_not_found(target: &String) -> String {
    format!("target [{}] not found!", target)
}

pub fn kill_player_attack(target: &Mob) -> String {
    format!("you attack {}!", target.label)
}

pub fn kill_mob_attack_someone(attacker: &Mob, target: &Mob) -> String {
    format!("{} attacks {}!", attacker.label, target.label)
}

pub fn kill_player_cancel(target: &Mob) -> String {
    format!("you relax, {} is not around", target.label)
}

pub fn kill_cancel(mob: &Mob, target: &Mob) -> String {
    format!("{} relax, {} is not around", mob.label, target.label)
}

pub fn kill_player_execute_attack(target: &Mob, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("you execute a attack and hit {} causing {} damage!", target.label, attack_result.damage)
    } else {
        format!("you execute a attack {} and miss!", target.label)
    }
}

pub fn kill_mob_execute_attack(mob: &Mob, target: &Mob, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("{} execute a attack and hit {} causing {} damage!", mob.label, target.label, attack_result.damage)
    } else {
        format!("{} execute a attack {} and miss!", mob.label, target.label)
    }
}
