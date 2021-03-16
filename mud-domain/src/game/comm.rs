use super::container::Container;
use super::domain::*;
use super::item::*;
use super::mob::*;
use crate::errors::{AsResult, Result};
use crate::game::astro_bodies::{AstroBodyKind, DistanceMkm};
use crate::game::obj::Obj;
use crate::game::outputs::OMarker;
use crate::game::prices::Money;
use crate::game::room::{Room, RoomId};
use crate::utils::text::{plot_points, PlotCfg, PlotPoint};
use commons::{ObjId, TotalTime, V2};
use logs::*;
use std::collections::{HashMap, HashSet};

pub struct PPMsg {
    pub private_msg: String,
    pub public_msg: String,
}

pub struct InventoryDesc<'a> {
    pub max_weight: Option<Weight>,
    pub total_weight: Weight,
    pub items: Vec<InventoryItemDesc<'a>>,
}

pub struct InventoryItemDesc<'a> {
    pub id: ItemId,
    pub label: &'a str,
    pub amount: u32,
    pub equipped: bool,
    pub weight: Option<f32>,
}

// TODO: move to a rule like system that control this semantic through
pub fn is_visible(container: &Container, obj_id: ObjId) -> bool {
    container.mobs.exists(obj_id)
        || container.items.exists(obj_id)
        || container.ships.exists(obj_id)
        || container.extractables.exist(obj_id)
}

pub fn get_visible_objects(container: &Container, mob_id: MobId, room_id: RoomId) -> Vec<ObjId> {
    container
        .locations
        .list_at(room_id)
        .filter(|id| *id != mob_id)
        .filter(|id| is_visible(container, *id))
        .collect()
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
  hire               - hire someone
  map                - show map of current zone
  buy <item>         - list objecst to buy or buy a item
  sell <item>        - list objecst to sell or sell a item
  extract <obj>      - extract resources from a stuff that can be extracted
-------------------------------------------------------------"#;

    str.to_string()
}

pub fn look_description(
    room_label: &str,
    room_desc: &str,
    exits: Vec<Dir>,
    can_exit: bool,
    visible_objects: Vec<&str>,
) -> Result<String> {
    let mut buffer = vec![];

    let mut exit_list = exits.iter().map(|dir| dir.as_str()).collect::<Vec<&str>>();

    if can_exit {
        exit_list.push("exit");
    }

    let exits = exit_list.join(", ");

    buffer.push(format!("[{}] - {}", room_label, exits));
    buffer.push(format!("{}", room_desc));

    for label in visible_objects {
        buffer.push(format!("- {}", label));
    }

    Ok(buffer.join("\n"))
}

pub fn unknown_input(input: &str) -> String {
    format!("unknown command '{}'", input)
}

pub fn say_you_say(msg: &str) -> String {
    format!("you say '{}{}{}'", OMarker::Literal, msg, OMarker::Reset)
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
    format!("now it is {}s after start", time.as_seconds_f64())
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
            "you attack {} and hit, causing {}/{} damage!",
            target, attack_result.damage_deliver, attack_result.damage_total
        )
    } else {
        format!("you attack {} and miss!", target)
    }
}

pub fn kill_mob_execute_attack(mob: &str, target: &str, attack_result: &AttackResult) -> String {
    if attack_result.success {
        format!(
            "{} execute a attack and hit {} causing {}/{} damage!",
            mob, target, attack_result.damage_deliver, attack_result.damage_total
        )
    } else {
        format!("{} execute a attack {} and miss!", mob, target)
    }
}

pub fn killed_by_player(mob: &str, xp: Xp) -> String {
    format!("you killed {} and receive {} XP", mob, xp)
}

pub fn killed(mob: &str) -> String {
    format!("{} was killed", mob)
}

pub fn kill_return_attack_self(aggressor_mob_label: &str) -> String {
    format!("You return combat against {}", aggressor_mob_label)
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

pub fn stats(xp: Xp, attributes: &Attributes, inventory: &InventoryDesc) -> String {
    let inventory_str = show_inventory(inventory);

    format!(
        "Stats: \n\
         attack:  {}\n\
         defense: {}\n\
         damage:  {}-{}\n\
         pv:      {}-{}\n\
         xp:      {}\n\
         {}\n",
        attributes.attack,
        attributes.defense,
        attributes.damage.min,
        attributes.damage.max,
        attributes.pv.current,
        attributes.pv.max,
        xp,
        inventory_str
    )
}

pub fn examine_target_not_found(target: &str) -> String {
    format!("no [{}] can be found!\n", target)
}

pub fn examine_target(
    mob_label: &str,
    xp: Xp,
    attributes: &Attributes,
    inventory: &InventoryDesc,
) -> String {
    format!(
        "you examine {}!\n{}",
        mob_label,
        stats(xp, attributes, inventory)
    )
}

pub fn examine_target_item(item: &str, inventory: &InventoryDesc) -> String {
    format!("you examine {}!\n{}", item, show_inventory(inventory))
}

pub fn show_inventory(inventory: &InventoryDesc) -> String {
    let mut buffer: Vec<String> = vec!["Inventory:".to_string()];
    for item in &inventory.items {
        buffer.push(format!(
            "- {}",
            print_item(item.label, item.amount, item.equipped)
        ));
    }

    if let Some(max_weight) = inventory.max_weight {
        buffer.push(format!("weight {}/{}", inventory.total_weight, max_weight));
    } else {
        buffer.push(format!("weight {}", inventory.total_weight));
    }

    buffer.join("\n")
}

fn print_item(item: &str, amount: u32, equipped: bool) -> String {
    let mut equip_str = "";
    let mut amount_str = "".to_string();
    if equipped {
        equip_str = "(equipped)";
    }
    if amount > 1 {
        amount_str = format!(" x{}", amount);
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

pub fn pick_fail_inventory_full(target_item: &str) -> String {
    format!("you can not pick {}, you already full", target_item)
}

pub fn inventory_full() -> String {
    format!("your inventory is full")
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
    format!(
        "can not enter at [{}], candidates: [{:?}]",
        label,
        candidates.join(", ")
    )
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

// TODO: add distance
pub fn space_show_move_targets(desc: &Vec<ShowSectorTreeBody>) -> String {
    let mut buffer = vec!["Targets:".to_string()];

    let items: Vec<String> = desc
        .iter()
        .enumerate()
        .flat_map(|(_i, desc)| Some(format!("- {}", desc.label)))
        .collect();

    buffer.extend(items);
    buffer.join("\n")
}

pub fn space_invalid_not_in_craft() -> String {
    "You can not do this, you are not in a ship.".to_string()
}

pub fn space_move() -> String {
    "command accepted, the ship is accelerating to the target".to_string()
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
    "you are not in a ship".to_string()
}

pub fn space_needs_to_be_in_space() -> String {
    "you need to be in space to do it".to_string()
}

pub fn space_land_started() -> String {
    "Landing procedures starting, reverting the ship and waiting for retro-burn position"
        .to_string()
}

pub fn space_land_retroburn() -> String {
    "retro-burn started, reducing our PE to landing point, engage engines!".to_string()
}

pub fn space_land_deorbit() -> String {
    "retro-burn complete, turning ship forward and descending to aero braking altitude".to_string()
}

pub fn space_land_aerobraking() -> String {
    "aerobraking to low altitude speeds".to_string()
}

pub fn space_land_approach() -> String {
    "approaching the landing pad".to_string()
}

pub fn space_land_landing() -> String {
    "starting landing procedures, preparing for touch down".to_string()
}

pub fn space_land_complete() -> String {
    "landing complete".to_string()
}

pub fn space_land_complete_others(craft_label: &str) -> String {
    format!("The {} come from orbit and land", craft_label)
}

pub fn space_launch_prepare() -> String {
    "preparing for launch".to_string()
}

pub fn space_launch_ignition() -> String {
    "ignite main engines, you fell the pressure while ship burns up".to_string()
}

pub fn space_launch_ascending() -> String {
    "burning horizontally to acquire horizontal speed".to_string()
}

pub fn space_launch_burning_circularization() -> String {
    "desired AP reached, circularizing the orbit".to_string()
}

pub fn space_launch_failed() -> String {
    "fail to launch, you can not launch from here".to_string()
}

pub fn space_launch_complete() -> String {
    "launch complete, you are in space now into a stable orbit".to_string()
}

pub fn space_launch_complete_others(_craft_label: &str) -> String {
    "{} have launched into orbit".to_string()
}

#[derive(Debug, Clone, Copy)]
pub enum ShowSectorTreeBodyKind {
    BodyKind(AstroBodyKind),
    Unknown,
}

impl From<AstroBodyKind> for ShowSectorTreeBodyKind {
    fn from(kind: AstroBodyKind) -> Self {
        ShowSectorTreeBodyKind::BodyKind(kind)
    }
}

#[derive(Clone, Debug)]
pub struct ShowSectorTreeBodyOrbit {
    pub orbit_id: ObjId,
    pub distance: DistanceMkm,
}

#[derive(Clone, Debug)]
pub struct ShowSectorTreeBody<'a> {
    pub id: ObjId,
    pub label: &'a str,
    pub orbit_id: ObjId,
    pub orbit_distance: DistanceMkm,
    pub kind: ShowSectorTreeBodyKind,
    pub is_self: bool,
}

pub fn show_sectortree<'a>(
    origin_id: ObjId,
    sector_label: &str,
    bodies: &'a Vec<ShowSectorTreeBody<'a>>,
) -> String {
    fn append<'a>(
        bodies: &'a Vec<ShowSectorTreeBody<'a>>,
        buffer: &mut Vec<String>,
        origin_id: ObjId,
        orbit_id: ObjId,
        prefix: &str,
    ) {
        let list = bodies.iter().filter(|e| e.orbit_id == orbit_id);

        let (local_prefix, next_prefix) = if orbit_id == origin_id {
            ("", "- ".to_string())
        } else {
            (prefix, format!("  {}", prefix))
        };

        for body in list {
            let highlight_str = if body.is_self { " <" } else { "" };
            buffer.push(format!(
                "{}{} {:.2}{}",
                local_prefix, body.label, body.orbit_distance, highlight_str
            ));
            append(bodies, buffer, origin_id, body.id, next_prefix.as_str());
        }
    }

    let mut buffer = Vec::new();
    let title = format!("[{}]", sector_label);
    buffer.push(title);
    append(bodies, &mut buffer, origin_id, origin_id, "");
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

pub fn space_fly_start() -> String {
    "orbital alignment started".to_string()
}

pub fn space_fly_alignment_complete_start_ejection_burn() -> String {
    "orbital alignment complete, starting ejection burn".to_string()
}

pub fn space_fly_ejection_burn_complete() -> String {
    "ejection burn complete, drifting in space".to_string()
}

pub fn space_fly_drift_complete() -> String {
    "approaching target, starting retro burn".to_string()
}

pub fn space_fly_retroburn_complete_start_orbital_sync() -> String {
    "retro burn complete, doing orbital alignment and circularizing".to_string()
}

pub fn space_fly_complete() -> String {
    "command complete".to_string()
}

#[derive(Debug)]
pub struct VendorTradeItemDisplay<'a> {
    pub label: &'a str,
    pub to_buy: Option<Money>,
    pub to_sell: Option<Money>,
}

pub fn vendor_operation_fail() -> String {
    "you have no vendor to interact, go away".to_string()
}

pub fn vendor_sell_item_not_found(label: &str) -> String {
    format!("you don't have a '{}' to sell", label)
}

pub fn vendor_sell_item_fail_has_no_price(label: &str) -> String {
    format!("you can not sell {}", label)
}

pub fn vendor_sell_item(label: &str, value: Money) -> String {
    format!("you sell {} and receive {}", label, value.as_u32())
}

pub fn vendor_sell_item_for_others(mob: &str, label: &str) -> String {
    format!("{} sell a {}", mob, label)
}

pub fn vendor_can_not_sell(label: &str) -> String {
    format!("'{}' can not be sell", label)
}

// TODO: use column based display
pub fn vendor_list(list: Vec<VendorTradeItemDisplay>) -> String {
    let mut buffer = String::new();
    buffer.push_str("List\n");
    buffer.push_str("name buy sell\n");

    for item in list {
        buffer.push_str("- ");
        buffer.push_str(item.label);
        buffer.push_str(" ");
        if let Some(buy_price) = item.to_buy {
            buffer.push_str(&buy_price.as_u32().to_string());
        } else {
            buffer.push_str("XXX");
        }
        buffer.push_str(" ");
        if let Some(sell_price) = item.to_sell {
            buffer.push_str(&sell_price.as_u32().to_string());
        } else {
            buffer.push_str("XXX");
        }
        buffer.push_str("\n");
    }

    buffer
}

pub fn vendor_buy_fail() -> String {
    format!("buy item fail")
}

pub fn vendor_buy_item_not_found(label: &str) -> String {
    format!("seller don't have a '{}'", label)
}

pub fn vendor_buy_you_have_not_enough_money(money: Money, price: Money) -> String {
    format!(
        "not enough money, it cost {} and you have only {}",
        price.as_u32(),
        money.as_u32()
    )
}

pub fn vendor_buy_success(item: &str, price: Money, new_money: Money) -> String {
    format!(
        "you bought a {} for {}, you have now {} of money",
        item,
        price.as_u32(),
        new_money.as_u32()
    )
}

pub fn vendor_buy_success_floor(
    item: &str,
    price: Money,
    new_money: Money,
    item_weight: Weight,
    available_weight: Weight,
) -> String {
    format!(
        "you bought a {} for {}, you have now {} of money, it's weight {} but you can only take {}, it was drop in the floor, ",
        item,
        price.as_u32(),
        new_money.as_u32(),
        item_weight,
        available_weight
    )
}

pub fn vendor_buy_success_others(mob_label: &str, item_label: &str) -> String {
    format!("{} bought a {}", mob_label, item_label)
}

pub fn vendor_buy_success_floor_others(mob_label: &str, item_label: &str) -> String {
    format!(
        "{} bought a {} and drop in the floor",
        mob_label, item_label
    )
}

pub fn hire_fail() -> String {
    "you can not hire now".to_string()
}
pub fn hire_fail_not_found(label: &str) -> String {
    format!("you can not hire {}", label)
}

pub fn hire(hired_label: &str) -> String {
    format!("{} hired", hired_label)
}

pub fn hire_others(mob_label: &str, hired_label: &str) -> String {
    format!("{} hired {}", mob_label, hired_label)
}

pub fn hire_list(candidates: Vec<&str>) -> String {
    let mut buff = "You can hire the following:\n".to_string();
    for candidate in candidates {
        buff.push_str(" - ");
        buff.push_str(candidate);
        buff.push_str("\n");
    }
    buff
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RoomMapCell {
    Empty,
    Room(ObjId),
    DoorHor,
    DoorVer,
}

#[derive(Debug, Clone)]
pub struct RoomMap {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<RoomMapCell>,
    pub portals_up: HashSet<ObjId>,
    pub portals_down: HashSet<ObjId>,
}

pub fn print_room_map(
    current_id: ObjId,
    room_map: RoomMap,
    labels: &HashMap<ObjId, String>,
) -> String {
    let mut buffer = String::new();

    buffer.push_str("Map\n");

    let mut room_indexes = vec![];
    let mut current = 0;

    let mut index = 0;
    for _y in 0..room_map.height {
        for _x in 0..room_map.width {
            match &room_map.cells[index] {
                RoomMapCell::Empty => buffer.push_str(".."),
                RoomMapCell::Room(obj_id) => {
                    let label = labels.get(obj_id).unwrap().as_str();

                    let index = room_indexes.len();
                    let to_up = room_map.portals_up.contains(obj_id);
                    let to_down = room_map.portals_down.contains(obj_id);
                    room_indexes.push((label, to_up, to_down));

                    if *obj_id == current_id {
                        current = index;
                        buffer.push_str("**");
                    } else {
                        buffer.push_str(format!("{:02}", index).as_str());
                    }
                }
                RoomMapCell::DoorHor => buffer.push_str("=="),
                RoomMapCell::DoorVer => buffer.push_str("||"),
            }

            index += 1;
        }
        buffer.push_str("\n");
    }

    buffer.push_str("labels:\n");

    // buffer.push_str(format!("** - {}\n", current_label).as_str());

    for index in 0..room_indexes.len() {
        let (label, to_up, to_down) = room_indexes[index];

        let up_down_str = match (to_up, to_down) {
            (true, true) => " <>",
            (true, false) => " >",
            (false, true) => " <",
            _ => "",
        };

        let index_label = if index == current {
            "**".to_string()
        } else {
            format!("{:02}", index)
        };

        buffer.push_str(format!("{:02} - {}{}\n", index_label, label, up_down_str).as_str());
    }

    buffer
}

pub fn space_jump_failed() -> String {
    "jump fail!".to_string()
}

pub fn space_jump_start() -> String {
    "starting jump procedures".to_string()
}
pub fn space_jump_recharging_capacitors() -> String {
    "recharging capacitors".to_string()
}
pub fn space_jump_do() -> String {
    "jumping".to_string()
}
pub fn space_jump_complete() -> String {
    "jump complete!".to_string()
}

pub fn extract_start(target: &str) -> String {
    format!("you start to extract {}", target)
}

pub fn extract_start_others(mob: &str, target: &str) -> String {
    format!("{} start to extract {}", mob, target)
}

pub fn extract_fail(target: &str) -> String {
    format!("you fail to extract {}", target)
}

pub fn extract_success(mob: &str, target: &str, item: &str) -> PPMsg {
    PPMsg {
        private_msg: format!("you extract {} from {}", item, target),
        public_msg: format!("{} extract {} from {}", mob, item, target),
    }
}

pub fn extract_stop(mob: &str, target: &str) -> PPMsg {
    PPMsg {
        private_msg: format!("you stop to extract from {}", target),
        public_msg: format!("{} stop to extract from {}", mob, target),
    }
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
                id: ObjId(1),
                label: "Sun",
                orbit_id: ObjId(0),
                orbit_distance: 0.0,
                kind: AstroBodyKind::Star.into(),
                is_self: false,
            },
            ShowSectorTreeBody {
                id: ObjId(2),
                label: "Earth",
                orbit_id: ObjId(1),
                orbit_distance: 2.0,
                kind: AstroBodyKind::Planet.into(),
                is_self: false,
            },
            ShowSectorTreeBody {
                id: ObjId(3),
                label: "Moon",
                orbit_id: ObjId(2),
                orbit_distance: 0.4,
                kind: AstroBodyKind::Planet.into(),
                is_self: false,
            },
            ShowSectorTreeBody {
                id: ObjId(4),
                label: "Asteroids",
                orbit_id: ObjId(1),
                orbit_distance: 80.0,
                kind: AstroBodyKind::AsteroidField.into(),
                is_self: false,
            },
            ShowSectorTreeBody {
                id: ObjId(5),
                label: "Ring",
                orbit_id: ObjId(3),
                orbit_distance: 0.05,
                kind: AstroBodyKind::Station.into(),
                is_self: false,
            },
            ShowSectorTreeBody {
                id: ObjId(6),
                label: "Light Cargo",
                orbit_id: ObjId(3),
                orbit_distance: 0.01,
                kind: AstroBodyKind::Ship.into(),
                is_self: true,
            },
        ];

        let sector_label = "Sector 1";
        let result = show_sectortree(ObjId(0), sector_label, &bodies);
        assert_eq!(
            result.as_str(),
            r##"[Sector 1]
Sun 0.00
- Earth 2.00
  - Moon 0.40
    - Ring 0.05
    - Light Cargo 0.01 <
- Asteroids 80.00
"##
        );
    }

    #[test]
    fn test_show_surface() {
        let desc = vec![
            SurfaceDesc {
                kind: ShowStarmapDescKind::Planet,
                pos: V2 { x: 5.0, y: 3.0 },
                me: false,
                label: "a planet".to_string(),
            },
            SurfaceDesc {
                kind: ShowStarmapDescKind::Craft,
                pos: V2 { x: 1.0, y: -2.0 },
                me: true,
                label: "A ship".to_string(),
            },
        ];
        let str = show_surface_map(&desc);
        assert_eq!("", str);
    }
}
