use super::item::*;
use super::domain::*;
use super::container::Container;
use super::mob::*;

use termion;
use commons::{TotalTime};
use std::collections::HashSet;

pub fn help() -> String {
    let str = r#"-------------------------------------------------------------
  [Help]
-------------------------------------------------------------
  look               - look around
  examine <target>   - examine target insides carefully
  n,s,e,w            - move to different directions
  say <msg>          - say something in the room
  uptime             - server uptime
  stats              - show your stats information and inventory
  rest               - rest to recovery from wounds, see stand
  stand              - sand up and stop to rest, see rest
  kill <target>      - attack something and try to kill it
  get <obj>          - pick up a <obj> from floor
  get <obj> in <obj> - pick up a <obj> from <from>
  equip <item>       - use a weapon or wear a armor
  remove <item>      - strip an item you are using
  drop <item>        - drop a object
  put <item> <obj>   - put a object into other container
-------------------------------------------------------------"#;

    str.to_string()
}

pub fn look_description(container: &Container, mob_id: MobId) -> Result<String, ()> {
    let room_id = container.locations.get(mob_id)?;
    let room = container.rooms.get(room_id)?;

    let exits: Vec<&str> = room.exits.iter()
        .map(|(dir, _)| dir.as_str())
        .collect();

    let exits = exits.join(", ");
    let mut mobs = vec![];
    let mut items = vec![];

    for obj_id in container.locations.list_at(room_id) {
        let item = container.items.get(obj_id);
        let mob = container.mobs.get(obj_id);

        match (mob, item) {
            (Ok(mob), _) => mobs.push(mob),
            (_, Ok(item)) => items.push(item),
            _ => {},
        }
    }

    let mobs =
        if mobs.is_empty() {
            "".to_string()
        } else {
            let labels: Vec<String> =
                mobs.iter()
                    .filter(|i| i.id != mob_id)
                    .map(|i| {
                        format!("- {} is here", i.label)
                    }).collect();

            labels.join("\n")
        };

    // TODO: migrate to location
    let items: Vec<String> = items
        .into_iter()
        .map(|item| format!("- {} in the floor", item.label))
        .collect();
    let items = items.join("\n");

    Ok(format!("{}\n{}\n[{}]\n{}\n{}\n", room.label, room.desc, exits, mobs, items).to_string())
}

pub fn unknown_input(input: &str) -> String {
    format!("unknown command '{}'\n", input)
}

pub fn say_you_say(msg: &str) -> String {
    format!("you say '{}'\n", msg)
}

pub fn say_someone_said(actor: &String, msg: &str) -> String {
    format!("{} says '{}'\n", actor, msg)
}

pub fn move_you_move(dir: &Dir) -> String {
    format!("you move to {}!", dir)
}

pub fn move_come(who: &str, dir: &Dir) -> String {
    format!("{} comes from {}.\n", who, dir)
}

pub fn move_goes(who: &str, dir: &Dir) -> String {
    format!("{} goes to {}.\n", who, dir)
}

pub fn move_not_possible(dir: &Dir) -> String {
    format!("not possible to move to {}!\n", dir)
}

pub fn spawn_mob(mob: &Mob) -> String {
    format!("a {} appears here from no where\n", mob.label)
}

pub fn uptime(time: TotalTime) -> String {
    format!("now it is {}s after start\n", time.as_f64())
}

pub fn kill_target_not_found(target: &str) -> String {
    format!("target [{}] not found!\n", target)
}

pub fn kill_can_not_kill_players(target: &str) -> String {
    format!("target [{}] is friendly, you can not kill him!\n", target)
}

pub fn attack_player_initiate(target: &Mob) -> String {
    format!("you attack {}!\n", target.label)
}

pub fn attack_mob_initiate_attack(attacker: &Mob, target: &Mob) -> String {
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
        format!("you attack {} and hit, causing {} damage!\n", target.label, attack_result.damage)
    } else {
        format!("you attack {} and miss!\n", target.label)
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

pub fn stats(mob: &Mob, inventory: &Vec<&Item>, equiped: &HashSet<ItemId>) -> String {
    let inventory_str = show_inventory(inventory, equiped);

    format!("Stats: \n\
        attack:  {}\n\
        defense: {}\n\
        damage:  {}-{}\n\
        pv:      {}-{}\n\
        {}\n",
            mob.attributes.attack,
            mob.attributes.defense,
            mob.attributes.damage.min,
            mob.attributes.damage.max,
            mob.attributes.pv.current,
            mob.attributes.pv.max,
            inventory_str
    )
}

pub fn examine_target_not_found(target: &str) -> String {
    format!("no [{}] can be found!\n", target)
}

pub fn examine_target(mob: &Mob, inventory: &Vec<&Item>, equiped: &HashSet<ItemId>) -> String {
    format!("you examine {}!\n{}", mob.label, stats(&mob, inventory, equiped))
}

pub fn examine_target_item(item: &Item, inventory: &Vec<&Item>) -> String {
    format!("you examine {}!\n{}", item.label, show_inventory(inventory, &HashSet::new()))
}

pub fn show_inventory(inventory: &Vec<&Item>, equiped: &HashSet<ItemId>) -> String {
    let mut buffer: Vec<String> = vec![
        "Inventory:".to_string(),
    ];
    for item in inventory {
        if equiped.contains(&item.id) {
            buffer.push(format!("- {}*", print_item(item)));
        } else {
            buffer.push(format!("- {}", print_item(item)));
        }
    }
    buffer.join("\n")
}

fn print_item(item: &&Item) -> String {
    if item.amount == 1 {
        format!("{}", item.label)
    } else {
        format!("{} ({})", item.label, item.amount)
    }
}

pub fn pick_where() -> String {
    "where? pick where?\n".to_string()
}

pub fn pick_where_not_found(target: &str) -> String {
    format!("there is no {} here, what are you talking about?\n", target)
}

pub fn pick_what() -> String {
//    let mut buffer: Vec<String> = vec![
//        "what do you want to pick?".to_string(),
//    ];
//    for item in items {
//        buffer.push(format!("- {}", print_item(item)));
//    }
//    buffer.join("\n")
    "get what?\n".to_string()
}

pub fn pick_player_from(target_inventory: &str, target_item: &str) -> String {
    format!("you pick a {} from {}", target_item, target_inventory)
}

pub fn pick_from(actor: &str, target_inventory: &str, target_item: &str) -> String {
    format!("{} pick a {} from {}", actor, target_item, target_inventory)
}

pub fn pick_player_from_room(target_item: &str) -> String {
    format!("you pick a {} from the floor", target_item)
}

pub fn pick_from_room(actor: &str, target_item: &str) -> String {
    format!("{} pick a {} from the floor", actor, target_item)
}

pub fn pick_fail_item_is_stuck(target_item: &str) -> String {
    format!("you can not get a {}", target_item)
}

pub fn pick_fail_storage_is_not_inventory(target_item: &str) -> String {
    format!("you can not get it, {} is not a storage", target_item)
}

pub fn equip_what() -> String {
    "what? what do you want to equip?".to_string()
}

pub fn equip_item_not_found(label: &str) -> String {
    format!("you can not find a {} to equip", label)
}

pub fn equip_item_invalid(label: &str) -> String {
    format!("you can not equip a {}\n", label)
}

pub fn equip_player_from_room(target_item: &str) -> String {
    format!("you equip a {}\n", target_item)
}

pub fn equip_from_room(actor: &str, target_item: &str) -> String {
    format!("{} equip a {}\n", actor, target_item)
}

pub fn strip_what() -> String {
    "what? what do you want to remove?\n".to_string()
}

pub fn strip_item_not_found(label: &str) -> String {
    format!("you can not find a {} to strip\n", label)
}

pub fn strip_player_from_room(target_item: &str) -> String {
    format!("you stop to use {}\n", target_item)
}

pub fn strip_from_room(actor: &str, target_item: &str) -> String {
    format!("{} stop to use {}\n", actor, target_item)
}

pub fn drop_item_no_target() -> String {
    "what do you want to drop?\n".to_string()
}

pub fn drop_item_not_found(label: &str) -> String {
    format!("you can not find a {} to drop\n", label)
}

pub fn drop_item(item_label: &str) -> String {
    format!("you drop a {}\n", item_label)
}

pub fn drop_item_others(actor: &str, item_label: &str) -> String {
    format!("{} drop a {}\n", actor, item_label)
}

pub fn admin_invalid_command() -> String {
    format!("invalid admin command")
}

pub fn admin_suicide() -> String {
    format!("you committed suicide")
}

pub fn admin_suicide_others(label: &str) -> String {
    format!("{} committed suicide", label)
}

pub fn mob_you_resurrected() -> String {
    format!("you have resurrected!")
}

pub fn mob_resurrected(label: &str) -> String {
    format!("{} have resurrected!", label)
}

pub fn rest_fail_in_combat() -> String {
    "you can not rest, you are FIGHTING!".to_string()
}

pub fn rest_start() -> String {
    "you sit and rest".to_string()
}

pub fn rest_healing(current_hp: i32) -> String {
    format!("you are healing, current hp {}", current_hp)
}

pub fn rest_healed() -> String {
    format!("you feel fully healed")
}

pub fn rest_start_others(label: &str) -> String {
    format!("{} sit and rest", label)
}

pub fn stand_fail_not_resting() -> String {
    "you are already standing".to_string()
}

pub fn stand_up() -> String {
    "you stand up".to_string()
}

pub fn stand_up_others(label: &str) -> String {
    format!("{} stand up", label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use commons::{DeltaTime, ObjId};

    fn item_0_coins() -> Item {
        let mut item = Item::new(
            ObjId(0),
            ITEM_KIND_GOLD,
            "coins".to_string()
        );

        item.amount = 2;

        item
    }

    fn item_1_weapon() -> Item {
        let mut item = Item::new(
            ObjId(1),
            ITEM_KIND_UNDEFINED,
            "weapon".to_string()
        );

        item.weapon = Some(Weapon {
            damage_min: 1,
            damage_max: 2,
            reload: DeltaTime(1.0)
        });

        item
    }

    fn strip_colors(input: String) -> String {
        input
    }

    #[test]
    fn show_inventory_test() {
        let coins = item_0_coins();
        let weapon = item_1_weapon();
        let items = vec![&coins, &weapon];
        let equip : HashSet<ItemId> = vec![weapon.id].into_iter().collect();
        let string = show_inventory(&items, &equip);
        assert_eq!("Inventory:\n\
                    - coins (2)\n\
                    - weapon*", string);
    }

    #[test]
    fn help_test() {
        let result = help();
        assert!(result.len() > 0);
    }
}
