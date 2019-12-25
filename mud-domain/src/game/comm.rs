use super::container::Container;
use super::domain::*;
use super::item::*;
use super::mob::*;
use logs::*;
use crate::utils::text::{plot_points, PlotCfg, PlotPoint};
use crate::errors::{Result, AsResult};
use commons::{TotalTime, V2, ObjId};

pub struct InventoryDesc<'a> {
    pub id: ItemId,
    pub label: &'a str,
    pub amount: u32,
    pub equipped: bool,
}

pub fn is_visible(container: &Container, obj_id: ObjId) -> bool {
    container.mobs.exists(obj_id) ||
    container.items.exists(obj_id)
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
  enter <target>     - enter in something
  out                - get out of something
-------------------------------------------------------------"#;

    str.to_string()
}

pub fn look_description(container: &Container, mob_id: MobId) -> Result<String> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let room = container.rooms.get(room_id).as_result()?;

    let mut buffer = vec![];

    let exits = room
        .exits
        .iter()
        .map(|(dir, _)| dir.as_str())
        .collect::<Vec<&str>>()
        .join(", ");

    let room_label = container.labels.get(room.id).unwrap();
    buffer.push(format!("[{}] - {}", room_label.label, exits));
    buffer.push(format!("{}", room_label.desc));

    for obj_id in container.locations.list_at(room_id) {
        if obj_id == mob_id {
            continue;
        }

        if !is_visible(container, obj_id) {
            continue;
        }

        let label = match container.labels.get(obj_id) {
            Some(lab) => &lab.label,
            _ => continue,
        };

        buffer.push(format!("- {}", label));
    }

    Ok(buffer.join("\n"))
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
    format!("you move to {}!", dir.as_str())
}

pub fn move_come(who: &str, dir: &Dir) -> String {
    format!("{} comes from {}.", who, dir.as_str())
}

pub fn move_goes(who: &str, dir: &Dir) -> String {
    format!("{} goes to {}.", who, dir.as_str())
}

pub fn move_not_possible(dir: &Dir) -> String {
    format!("not possible to move to {}!", dir.as_str())
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
        format!(
            "you attack {} and hit, causing {} damage!",
            target, attack_result.damage
        )
    } else {
        format!("you attack {} and miss!", target)
    }
}

pub fn kill_mob_execute_attack(mob: &str, target: &str, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!(
            "{} execute a attack and hit {} causing {} damage!",
            mob, target, attack_result.damage
        )
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
    format!(
        "{} give back combat against {}",
        mob_label, aggressor_mob_label
    )
}

pub fn item_corpse_appears_in_room(item: &str) -> String {
    format!("{} appears here!", item)
}

pub fn item_body_disappears(item: &str) -> String {
    format!("a {} disappear.", item)
}

pub fn stats(attributes: &Attributes, inventory: &Vec<InventoryDesc>) -> String {
    let inventory_str = show_inventory(inventory);

    format!(
        "Stats: \n\
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

pub fn examine_target(
    mob_label: &str,
    attributes: &Attributes,
    inventory: &Vec<InventoryDesc>,
) -> String {
    format!(
        "you examine {}!\n{}",
        mob_label,
        stats(attributes, inventory)
    )
}

pub fn examine_target_item(item: &str, inventory: &Vec<InventoryDesc>) -> String {
    format!("you examine {}!\n{}", item, show_inventory(inventory))
}

pub fn show_inventory(inventory: &Vec<InventoryDesc>) -> String {
    let mut buffer: Vec<String> = vec!["Inventory:".to_string()];
    for item in inventory {
        buffer.push(format!(
            "- {}",
            print_item(item.label, item.amount, item.equipped)
        ));
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

pub fn enter_fail() -> String {
    "fail to enter, you don't understand whats happens".to_string()
}

pub fn enter_player(target: &str) -> String {
    format!("you enter in the {}", target).to_string()
}

pub fn enter_others(mob: &str, target: &str) -> String {
    format!("{} enter in the {}", mob, target).to_string()
}

pub fn enter_others_other_side(mob: &str) -> String {
    format!("{} comes from outside", mob).to_string()
}

pub fn enter_invalid(label: &str, candidates: &Vec<&str>) -> String {
    format!("can not enter at [{}], candidates: [{:?}]", label, candidates.join(", "))
}

pub fn enter_list(candidates: &Vec<&str>) -> String {
    format!("valid locations to enter: [{:?}]", candidates.join(", "))
}

pub fn out_fail() -> String {
    "you can not go out from here".to_string()
}

pub fn out_fail_bad_outside() -> String {
    "you can not go out from here, outside is not safe".to_string()
}

pub fn out_player() -> String {
    "you go outside".to_string()
}

pub fn out_others_other_side(mob: &str, target: &str) -> String {
    format!("{} come out of the {}", mob, target).to_string()
}

pub fn out_others(mob: &str) -> String {
    format!("{} goes outside", mob).to_string()
}

pub enum ShowStarmapDescKind {
    Planet,
    Craft,
}

pub struct SurfaceDesc {
    pub kind: ShowStarmapDescKind,
    pub pos: V2,
    pub me: bool,
    pub label: String,
}

pub fn show_surface_map(desc: &Vec<SurfaceDesc>) -> String {
    let cfg = PlotCfg {
        width: 10,
        height: 10,
        min_scale: 1.0,
    };

    let mut content_table = vec![];

    let points = desc
        .iter()
        .enumerate()
        .map(|(i, desc)| {
            let ch = match desc.kind {
                ShowStarmapDescKind::Craft if desc.me => '@'.to_string(),
                ShowStarmapDescKind::Craft => '%'.to_string(),
                ShowStarmapDescKind::Planet => 'O'.to_string(),
            };

            content_table.push(format!("{} - {} {}", i, ch, desc.label));

            PlotPoint {
                x: desc.pos.x,
                y: desc.pos.y,
                c: ch,
            }
        })
        .collect();

    let map = plot_points(&cfg, &points);
    let mut buffer: Vec<String> = map.into_iter().map(|i| i.join("")).collect();

    buffer.push("\n".to_string());
    buffer.append(&mut content_table);

    buffer.join("\n")
}

pub fn space_show_move_targets(desc: &Vec<SurfaceDesc>) -> String {
    let mut buffer = vec!["Targets:".to_string()];

    let items: Vec<String> = desc
        .iter()
        .enumerate()
        .flat_map(|(_i, desc)| {
            if desc.me {
                return None;
            }

            Some(format!("- {}", desc.label))
        })
        .collect();

    buffer.extend(items);
    buffer.join("\n")
}

pub fn space_invalid_not_in_craft() -> String {
    "You can not do this, you are not in a craft.".to_string()
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

pub fn space_command_complete() -> String {
    "command complete!".to_string()
}

pub fn space_land_invalid() -> String {
    "invalid selection, can not land on that".to_string()
}

pub fn space_not_in_craft() -> String {
    "you are not in a craft".to_string()
}

pub fn space_needs_to_be_in_space() -> String {
    "you need to be in space to do it".to_string()
}

pub fn space_land_complete() -> String {
    "landing complete".to_string()
}

pub fn space_land_complete_others(_craft_label: &str) -> String {
    "The {} come from orbit and land".to_string()
}

pub fn space_launch_failed() -> String {
    "fail to launch, you can not launch from here".to_string()
}

pub fn space_launch_complete() -> String {
    "launch complete, you are in space now".to_string()
}

pub fn space_launch_complete_others(_craft_label: &str) -> String {
    "{} have launched into orbit".to_string()
}

#[derive(Debug, Clone, Copy)]
pub enum ShowSectorTreeBodyKind {
    Planet,
    Star,
    Ship,
    Asteroids,
    Station,
    Unknown,
}

#[derive(Debug)]
pub struct ShowSectorTreeBody<'a> {
    pub id: ObjId,
    pub label: &'a str,
    pub orbit_id: Option<ObjId>,
    pub kind: ShowSectorTreeBodyKind,
}

pub fn show_sectortree<'a>(bodies: &'a Vec<ShowSectorTreeBody<'a>>) -> String {
    fn append<'a>(bodies: &'a Vec<ShowSectorTreeBody<'a>>, buffer: &mut Vec<String>, orbit_id: Option<ObjId>, prefix: &str) {
        let list = bodies
            .iter()
            .filter(|e| e.orbit_id == orbit_id);

        let (local_prefix, next_prefix) =
            if orbit_id.is_none() {
                ("", "- ".to_string())
            } else {
                (prefix, format!("  {}", prefix))
            };

        for body in list {
            buffer.push(format!("{}{}", local_prefix, body.label));
            append(bodies, buffer, Some(body.id), next_prefix.as_str());
        }
    }

    let mut buffer = Vec::new();
    append(bodies, &mut buffer, None, "");
    buffer.push("".to_string());

    buffer.join("\n")
}

//pub fn space_land_started() -> String {
//    "command accepted, starting landing procedures.".to_string()
//}

pub fn space_land_list(candidates: &Vec<&str>) -> String {
    let mut buffer = Vec::new();
    buffer.push("Landing locations:".to_string());

    for label in candidates {
        buffer.push(format!("- {}", label))
    }
    buffer.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use commons::V2;

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

        let string = show_surface_map(&objects);
        //        assert_eq!("", string.as_str());
        assert!(string.as_str().contains("2 - @ three"));
    }

    #[test]
    fn test_show_sectortree() {
        let bodies = vec![
            ShowSectorTreeBody {
                id: ObjId(0),
                label: "Sun",
                orbit_id: None,
                kind: ShowSectorTreeBodyKind::Star
            },
            ShowSectorTreeBody {
                id: ObjId(1),
                label: "Earth",
                orbit_id: Some(ObjId(0)),
                kind: ShowSectorTreeBodyKind::Planet
            },
            ShowSectorTreeBody {
                id: ObjId(2),
                label: "Moon",
                orbit_id: Some(ObjId(1)),
                kind: ShowSectorTreeBodyKind::Planet
            },
            ShowSectorTreeBody {
                id: ObjId(3),
                label: "Asteroids",
                orbit_id: Some(ObjId(0)),
                kind: ShowSectorTreeBodyKind::Asteroids
            },
            ShowSectorTreeBody {
                id: ObjId(4),
                label: "Ring",
                orbit_id: Some(ObjId(2)),
                kind: ShowSectorTreeBodyKind::Ship
            }
        ];
        let result = show_sectortree(&bodies);
        assert_eq!(result.as_str(), r##"Sun
- Earth
  - Moon
    - Ring
- Asteroids
"##);
    }
}
