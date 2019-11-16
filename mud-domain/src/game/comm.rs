use super::item::*;
use super::domain::*;
use super::container::Container;
use super::mob::*;
use commons::{TotalTime, V2};
use crate::utils::text::{PlotPoint, PlotCfg, plot_points, mkstring, append_right};
use std::process::id;

pub struct InventoryDesc<'a> {
    pub id: ItemId,
    pub label: &'a str,
    pub amount: u32,
    pub equipped: bool,
}

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
                        let label = container.labels.get_label(i.id).unwrap_or("???");
                        format!("- {} is here", label)
                    }).collect();

            labels.join("\n")
        };

    // TODO: migrate to location
    let items: Vec<String> = items
        .into_iter()
        .map(|item| {
            let label = container.labels.get_label(item.id).unwrap_or("???");
            format!("- {} in the floor", label)
        })
        .collect();
    let items = items.join("\n");

    let room_label = container.labels.get(room.id).unwrap();

    Ok(format!("{}\n{}\n[{}]\n{}\n{}\n", room_label.label, room_label.desc, exits, mobs, items).to_string())
}

pub fn unknown_input(input: &str) -> String {
    format!("unknown command '{}'", input)
}

pub fn say_you_say(msg: &str) -> String {
    format!("you say '{}'", msg)
}

pub fn say_someone_said(actor: &str, msg: &str) -> String {
    format!("{} says '{}'", actor, msg)
}

pub fn move_you_move(dir: &Dir) -> String {
    format!("you move to {}!", dir)
}

pub fn move_come(who: &str, dir: &Dir) -> String {
    format!("{} comes from {}.", who, dir)
}

pub fn move_goes(who: &str, dir: &Dir) -> String {
    format!("{} goes to {}.", who, dir)
}

pub fn move_not_possible(dir: &Dir) -> String {
    format!("not possible to move to {}!", dir)
}

pub fn spawn_mob(label: &str) -> String {
    format!("a {} appears here from no where", label)
}

pub fn uptime(time: TotalTime) -> String {
    format!("now it is {}s after start", time.as_f64())
}

pub fn kill_target_not_found(target: &str) -> String {
    format!("target [{}] not found!", target)
}

pub fn kill_can_not_kill_players(target: &str) -> String {
    format!("target [{}] is friendly, you can not kill him!", target)
}

pub fn attack_player_initiate(label: &str) -> String {
    format!("you attack {}!", label)
}

pub fn attack_mob_initiate_attack(attacker: &str, target: &str) -> String {
    format!("{} attacks {}!", attacker, target)
}

pub fn kill_player_cancel(target: &str) -> String {
    format!("you relax, {} is not around", target)
}

pub fn kill_cancel(mob: &str, target: &str) -> String {
    format!("{} relax, {} is not around", mob, target)
}

pub fn kill_player_execute_attack(target: &str, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("you attack {} and hit, causing {} damage!", target, attack_result.damage)
    } else {
        format!("you attack {} and miss!", target)
    }
}

pub fn kill_mob_execute_attack(mob: &str, target: &str, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!("{} execute a attack and hit {} causing {} damage!", mob, target, attack_result.damage)
    } else {
        format!("{} execute a attack {} and miss!", mob, target)
    }
}

pub fn killed_by_player(mob: &str) -> String {
    format!("you killed {}.", mob)
}

pub fn killed(mob: &str) -> String {
    format!("{} was killed", mob)
}

pub fn kill_return_attack(mob_label: &str, aggressor_mob_label: &str) -> String {
    format!("{} give back combat against {}", mob_label, aggressor_mob_label)
}

pub fn item_body_appears_in_room(item: &str) -> String {
    format!("a {} appears here!", item)
}

pub fn item_body_disappears(item: &str) -> String {
    format!("a {} disappear.", item)
}

pub fn stats(attributes: &Attributes, inventory: &Vec<InventoryDesc>) -> String {
    let inventory_str = show_inventory(inventory);

    format!("Stats: \n\
        attack:  {}\n\
        defense: {}\n\
        damage:  {}-{}\n\
        pv:      {}-{}\n\
        {}\n",
            attributes.attack,
            attributes.defense,
            attributes.damage.min,
            attributes.damage.max,
            attributes.pv.current,
            attributes.pv.max,
            inventory_str
    )
}

pub fn examine_target_not_found(target: &str) -> String {
    format!("no [{}] can be found!\n", target)
}

pub fn examine_target(mob_label: &str, attributes: &Attributes, inventory: &Vec<InventoryDesc>) -> String {
    format!("you examine {}!\n{}", mob_label, stats(attributes, inventory))
}

pub fn examine_target_item(item: &str, inventory: &Vec<InventoryDesc>) -> String {
    format!("you examine {}!\n{}", item, show_inventory(inventory))
}

pub fn show_inventory(inventory: &Vec<InventoryDesc>) -> String {
    let mut buffer: Vec<String> = vec![
        "Inventory:".to_string(),
    ];
    for item in inventory {
        buffer.push(format!("- {}", print_item(item.label, item.amount, item.equipped)));
    }
    buffer.join("\n")
}

fn print_item(item: &str, amount: u32, equipped: bool) -> String {
    let mut equip_str = "";
    let mut amount_str = "".to_string();
    if equipped {
        equip_str = "*";
    }
    if amount > 1 {
        amount_str = format!("({})", amount);
    }
    format!("{}{}{}", item, equip_str, amount_str)
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

pub fn space_not_in_craft() -> String {
    "you are not in a craft".to_string()
}

pub fn space_needs_to_be_in_space() -> String {
    "you need to be in space to do it".to_string()
}

pub enum ShowStarmapDescKind {
    Planet,
    Craft
}

pub struct SurfaceDesc {
    pub kind: ShowStarmapDescKind,
    pub pos: V2,
    pub me: bool,
    pub label: String,
}

pub fn space_show_sectormap(desc: &Vec<SurfaceDesc>) -> String {
    let cfg = PlotCfg {
        width: 10,
        height: 10,
        min_scale: 1.0,
    };

    let mut content_table = vec![];

    let points = desc.iter().enumerate().map(|(i, desc)| {
        let ch = match desc.kind {
            ShowStarmapDescKind::Craft if desc.me => '@'.to_string(),
            ShowStarmapDescKind::Craft => '%'.to_string(),
            ShowStarmapDescKind::Planet => 'O'.to_string(),
        };

        content_table.push(format!("{} - {} {}", i, ch, desc.label));

        PlotPoint {
            x: desc.pos.x,
            y: desc.pos.y,
            c: ch
        }
    }).collect();

    let map = plot_points(&cfg, &points);
    let mut buffer: Vec<String> = map.into_iter().map(|i| {
        i.join("")
    }).collect();

    buffer.push("\n".to_string());
    buffer.append(&mut content_table);

    buffer.join("\n")
}

pub fn space_show_move_targets(desc: &Vec<SurfaceDesc>) -> String {
    let mut buffer = vec!["Targets:".to_string()];

    let items: Vec<String> =
        desc.iter().enumerate().flat_map(|(i, desc)| {
            if desc.me {
                return None;
            }

            Some(format!("- {}", desc.label))
        }).collect();

    buffer.extend(items);
    buffer.join("\n")
}

pub fn space_move() -> String {
    "command accepted, the craft is accelerating to the target".to_string()
}

pub fn space_move_invalid() -> String {
    "can not move to that".to_string()
}

pub fn space_command_failed() -> String {
    "command failed!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
//    use commons::{DeltaTime, ObjId};
//    use std::collections::HashSet;

//    fn item_0_coins() -> Item {
//        let mut item = Item::new(
//            ObjId(0),
//            ITEM_KIND_GOLD,
//            "coins".to_string()
//        );
//
//        item.amount = 2;
//
//        item
//    }
//
//    fn item_1_weapon() -> Item {
//        let mut item = Item::new(
//            ObjId(1),
//            ITEM_KIND_UNDEFINED,
//            "weapon".to_string()
//        );
//
//        item.weapon = Some(Weapon {
//            damage_min: 1,
//            damage_max: 2,
//            reload: DeltaTime(1.0)
//        });
//
//        item
//    }
//
//    fn strip_colors(input: String) -> String {
//        input
//    }

    #[test]
    fn show_inventory_test() {
//        let coins = item_0_coins();
//        let weapon = item_1_weapon();
//        let items = vec![&coins, &weapon];
//        let equip : HashSet<ItemId> = vec![weapon.id].into_iter().collect();
//        let string = show_inventory(&items, &equip);
//        assert_eq!("Inventory:\n\
//                    - coins (2)\n\
//                    - weapon*", string);
    }

    #[test]
    fn help_test() {
        let result = help();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_space_show_starmap() {
        let objects = vec![
            SurfaceDesc {
                kind: ShowStarmapDescKind::Planet,
                pos: V2::new(-2.0, 1.0),
                me: false,
                label: "one".to_string(),
            },
            SurfaceDesc {
                kind: ShowStarmapDescKind::Planet,
                pos: V2::new(1.0, 3.0),
                me: false,
                label: "two".to_string(),
            },
            SurfaceDesc {
                kind: ShowStarmapDescKind::Craft,
                pos: V2::new(1.0, 0.0),
                me: true,
                label: "three".to_string(),
            },
            SurfaceDesc {
                kind: ShowStarmapDescKind::Craft,
                pos: V2::new(2.0, -1.0),
                me: false,
                label: "four".to_string(),
            },
        ];

        let string = space_show_sectormap(&objects);
//        assert_eq!("", string.as_str());
        assert!(string.as_str().contains(".......... 2 - @ three"));
    }
}
