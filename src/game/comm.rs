use super::item::*;
use super::domain::*;
use super::container::Container;
use super::mob::*;

pub fn look_description(container: &Container, ctx: &PlayerCtx) -> String {
    let exits: Vec<String> = ctx.room.exits.iter()
        .map(|(dir, _)| dir.to_string())
        .collect();

    let exits = exits.join(", ");
    let mobs = container.mobs.search(Some(&ctx.avatar.room_id), None);
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

    let items: Vec<String> = container.items.list_at(&ctx.avatar.room_id)
        .iter()
        .map(|item| format!("- {} in the floor", item.label))
        .collect();
    let items = items.join("\n");

    format!("{}\n\n{}\n\n[{}]\n\n{}\n{}\n\n", ctx.room.label, ctx.room.desc, exits, mobs, items).to_string()
}

pub fn unknown_input(input: String) -> String {
    format!("unknown command '{}'\n", input)
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
    format!("a {} appears here from no where\n", mob.label)
}

pub fn uptime(time: &Seconds) -> String {
    format!("now it is {} seconds after start\n", time.0)
}

pub fn kill_target_not_found(target: &String) -> String {
    format!("target [{}] not found!\n", target)
}

pub fn kill_can_not_kill_players(target: &String) -> String {
    format!("target [{}] is friendly, you can not kill him!\n", target)
}

pub fn kill_player_attack(target: &Mob) -> String {
    format!("you attack {}!\n", target.label)
}

pub fn kill_mob_attack_someone(attacker: &Mob, target: &Mob) -> String {
    format!("{} attacks {}!\n", attacker.label, target.label)
}

pub fn kill_player_cancel(target: &Mob) -> String {
    format!("you relax, {} is not around\n", target.label)
}

pub fn kill_cancel(mob: &Mob, target: &Mob) -> String {
    format!("{} relax, {} is not around\n", mob.label, target.label)
}

pub fn kill_player_execute_attack(target: &Mob, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("you execute a attack and hit {} causing {} damage!\n", target.label, attack_result.damage)
    } else {
        format!("you execute a attack {} and miss!\n", target.label)
    }
}

pub fn kill_mob_execute_attack(mob: &Mob, target: &Mob, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("{} execute a attack and hit {} causing {} damage!\n", mob.label, target.label, attack_result.damage)
    } else {
        format!("{} execute a attack {} and miss!\n", mob.label, target.label)
    }
}

pub fn killed_by_player(mob: &Mob) -> String {
    format!("you killed {}.\n", mob.label)
}

pub fn killed(mob: &Mob) -> String {
    format!("{} was killed\n", mob.label)
}

pub fn kill_return_attack(mob_label: &String, aggressor_mob_label: &String) -> String {
    format!("{} give back combat against {}\n", mob_label, aggressor_mob_label)
}

pub fn item_body_appears_in_room(item: &Item) -> String {
    format!("a {} appears here!\n", item.label)
}

pub fn item_body_disappears(item: &Item) -> String {
    format!("a {} disappear.\n", item.label)
}

pub fn stats(mob: &Mob) -> String {
    format!("Stats: \n\
        attack:  {}\n\
        defense: {}\n\
        damage:  {}-{}\n\
        pv:      {}-{}\n",
            mob.attributes.attack,
            mob.attributes.defense,
            mob.attributes.damage.min,
            mob.attributes.damage.max,
            mob.attributes.pv.current,
            mob.attributes.pv.max
    )
}

pub fn examine_target_not_found(target: &String) -> String {
    format!("no [{}] can be found!\n", target)
}

pub fn examine_target(mob: &Mob, inventory: &Vec<&Item>) -> String {
    format!("you examine {}!\n{}\n{}", mob.label, stats(&mob), show_inventory(inventory))
}

pub fn show_inventory(inventory: &Vec<&Item>) -> String {
    let mut buffer: Vec<String> = vec![
        "Iventory:".to_string(),
    ];
    for item in inventory {
        if item.amount == 1 {
            buffer.push(format!("- {}", item.label));
        } else {
            buffer.push(format!("- {} ({})", item.label, item.amount));
        }
    }
    buffer.join("\n")
}
