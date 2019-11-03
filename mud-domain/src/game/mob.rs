use std::collections::HashMap;

use crate::game::body::create_body;
use crate::game::Ctx;
use commons::*;


use super::combat;
use super::comm;
use super::container::Container;
use super::domain::*;
use super::item::*;
use super::Outputs;
use super::room::RoomId;
use crate::game::obj::{ObjId, Objects};

// TODO: Move this to a injected configuration
// TODO: This is more Player related that mob
// TODO: now it get ugly with hardcoded ID
pub const ID_ROOM_INIT: RoomId = ObjId(0);

pub type MobId = ObjId;
pub type MobPrefabId = ObjId;

/// What mob should be doing
#[derive(Clone, Debug, Copy)]
pub enum MobCommand {
    None,
    Kill { target: MobId }
}

impl MobCommand {
    pub fn is_idle(&self) -> bool {
        match self {
            MobCommand::None => true,
            _ => false,
        }
    }
}

/// What is current doing
#[derive(Clone,Debug, PartialEq)]
pub enum MobAction {
    None,
    Combat,
    Resting
}

#[derive(Clone, Debug)]
pub struct Damage {
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Debug)]
pub struct Pv {
    pub current: i32,
    pub max: u32,
    pub heal_rate: DeltaTime,
}

impl Pv {
    pub fn is_damaged(&self) -> bool {
        self.current < self.max as i32
    }
}

#[derive(Clone, Debug)]
pub struct Attributes {
    pub attack: u32,
    pub defense: u32,
    pub damage: Damage,
    pub pv: Pv,
    pub attack_calm_down: DeltaTime,
}

#[derive(Clone, Debug)]
struct MobState {
    // after this total time can attack
    attack_calm_down: TotalTime,
    // after this total time can heal
    heal_calm_down: TotalTime,
    action: MobAction
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_calm_down: TotalTime(0.0),
            heal_calm_down: TotalTime(0.0),
            action: MobAction::None
        }
    }
}

#[derive(Clone, Debug)]
pub struct AttackResult {
    pub success: bool,
    pub damage: u32,
    pub attack_dice: u32,
    pub defense_dice: u32,
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: MobId,
    pub room_id: RoomId,
    pub label: String,
    pub is_avatar: bool,
    pub command: MobCommand,
    pub attributes: Attributes,
    state: MobState,
}

impl Mob {
    pub fn new(id: MobId, room_id: RoomId, label: String, attributes: Attributes) -> Self {
        Mob {
            id,
            room_id,
            label,
            is_avatar: false,
            command: MobCommand::None,
            attributes,
            state: MobState::new(),
        }
    }

    pub fn add_attack_calm_time(&mut self, total_time: TotalTime) {
        let next = TimeTrigger::next(self.attributes.attack_calm_down, total_time);
        self.state.attack_calm_down = next;
    }

    pub fn is_read_to_attack(&self, total_time: TotalTime) -> bool {
        let trigger = TimeTrigger::should_trigger(self.state.attack_calm_down, total_time);
        trigger
    }

    pub fn is_combat(&self) -> bool {
        self.state.action == MobAction::Combat
    }

    pub fn is_resting(&self) -> bool {
        self.state.action == MobAction::Resting
    }

    pub fn set_action(&mut self, action: MobAction, total: TotalTime) {
        self.state.action = action;

        match self.state.action {
            MobAction::Resting => {
                self.state.heal_calm_down = TimeTrigger::next(self.attributes.pv.heal_rate, total);
            },
            _ => {}
        }
    }

    pub fn update_resting(&mut self, total: TotalTime) -> bool {
        if !self.attributes.pv.is_damaged() {
            return false;
        }

        match TimeTrigger::check_trigger(self.attributes.pv.heal_rate, self.state.heal_calm_down, total) {
            Some(next) => {
                self.state.heal_calm_down = next;
                self.attributes.pv.current += 1;
                true
            },
            None => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MobPrefab {
    pub id: MobPrefabId,
    pub label: String,
    pub attributes: Attributes,
    pub inventory: Vec<ItemPrefabId>
}

pub struct MobRepository {
    index: HashMap<MobId, Mob>,
    mob_prefabs: HashMap<MobPrefabId, MobPrefab>,
}

impl MobRepository {
    pub fn new() -> Self {
        MobRepository {
            index: HashMap::new(),
            mob_prefabs: HashMap::new(),
        }
    }

    // TODO: iterator of ref?
    pub fn list(&self) -> Vec<MobId> {
        self.index
            .iter()
            .into_iter()
            .map(| (id, _)| id.clone())
            .collect()
    }

    pub fn add(&mut self, mob: Mob) -> &Mob {
        if self.exists(mob.id) {
            panic!("mob already exists")
        }
        let id = mob.id;
        self.index.insert(id, mob);
        self.index.get(&id).unwrap()
    }

    pub fn update(&mut self, mob: Mob) -> &Mob {
        let id = mob.id;

        let old_mob = self.index.remove(&id);
        if old_mob.is_none() {
            panic!("mob do not exists")
        }

        self.index.insert(id, mob);
        self.index.get(&id).unwrap()
    }

    pub fn remove(&mut self, id: &MobId) {
        self.index.remove(&id);
    }

    pub fn get(&self, id: MobId) -> &Mob {
        self.index.get(&id).unwrap()
    }

    pub fn find(&self, id: &MobId) -> Option<&Mob> {
        self.index.get(id)
    }

    pub fn exists(&self, id: MobId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn search(&self, room_id: Option<RoomId>, name: Option<&str>) -> Vec<&Mob> {
        self.index
            .iter()
            .filter(|(_, mob)| {
                if let Some(room_id) = room_id {
                    if mob.room_id != room_id {
                        return false;
                    }
                }

                if let Some(name) = name {
                    if !mob.label.eq_ignore_ascii_case(&name) {
                        return false;
                    }
                }

                true
            })
            .map(|(_, mob)| mob)
            .collect()
    }

    pub fn set_mob_attack_target(&mut self, mob_id: MobId, target: MobId) {
        let mut mob = self.index.get_mut(&mob_id).unwrap();
        mob.command = MobCommand::Kill { target: target.clone() };
        mob.state.action = MobAction::Combat;
    }

    pub fn cancel_attack(&mut self, mob_id: MobId) {
        let mut mob = self.index.get_mut(&mob_id).unwrap();
        mob.command = MobCommand::None;
        mob.state.action = MobAction::None;
    }

    pub fn is_avatar(&self, id: &MobId) -> bool {
        self.index.get(id).unwrap().is_avatar
    }

    pub fn add_prefab(&mut self, mob_prefab: MobPrefab) {
        assert!(!self.mob_prefabs.contains_key(&mob_prefab.id));
        self.mob_prefabs.insert(mob_prefab.id, mob_prefab);
    }

    pub fn get_mob_prefab(&mut self, id: MobPrefabId) -> &MobPrefab {
        self.mob_prefabs.get(&id)
            .expect(format!("could not found mob prefab id {:?}", id).as_str())
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        use serde_json::json;
//
//        for (id, obj) in self.index.iter() {
//            let command_json = match obj.command {
//                MobCommand::None => json!({ "kind": "idle" }),
//                MobCommand::Kill { target } => json!({ "kind": "kill", "target": target.0 }),
//            };
//
//            let obj_json = json!({
//                "room_id": obj.room_id.0,
//                "label": obj.label,
//                "is_avatar": obj.is_avatar,
//                "command": command_json,
//                "attributes": {
//                    "attack": obj.attributes.attack,
//                    "defense": obj.attributes.defense,
//                    "damage_min": obj.attributes.damage.min,
//                    "damage_max": obj.attributes.damage.max,
//                    "damage_calm_down": obj.attributes.attack_calm_down.as_float(),
//                    "pv": obj.attributes.pv.current,
//                    "pv_max": obj.attributes.pv.max,
//                    "pv_heal_rate": obj.attributes.pv.heal_rate.as_float(),
//                },
//                "state": {
//                    "attack_ready": obj.state.attack_calm_down.as_float(),
//                    "heal_ready": obj.state.heal_calm_down.as_float(),
//                    "action": match obj.state.action {
//                        MobAction::None => "none",
//                        MobAction::Combat => "combat",
//                        MobAction::Resting => "rest",
//                    },
//                }
//            });
//
//            save.add(id.0, "mob", obj_json);
//        }
//    }
}

// TODO: move game rules with output outside of mobs module
pub fn run_tick(ctx: &mut Ctx) {
    for mob_id in ctx.container.mobs.list() {
        if !ctx.container.mobs.exists(mob_id) {
            continue;
        }

        let mob = ctx.container.mobs.get(mob_id);

        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                combat::tick_attack(ctx.container, ctx.outputs, mob_id, target);
            }
        }

        let mob = ctx.container.mobs.get(mob_id);
        if mob.is_resting() {
            let mut mob = mob.clone();
            if mob.update_resting(ctx.container.time.total) {
                if mob.is_avatar {
                    let player = ctx.container.players.find_player_from_avatar_mob_id(mob.id).unwrap();
                    if mob.attributes.pv.is_damaged() {
                        ctx.outputs.private(player.id, comm::rest_healing(mob.attributes.pv.current));
                    } else {
                        ctx.outputs.private(player.id, comm::rest_healed());
                    }
                }
            }
            ctx.container.mobs.update(mob);
        }
    }
}

// TODO: move game rules with output outside of mobs module
pub fn kill_mob(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    create_body(container, outputs, mob_id);

    // remove mob
    let mob = container.mobs.get(mob_id);
    if mob.is_avatar {
        respawn_avatar(container, outputs, mob_id);
    } else {
        container.mobs.remove(&mob_id);
    }
}

// TODO: move game rules with output outside of mobs module
pub fn respawn_avatar(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    let mut mob = container.mobs.get(mob_id).clone();
    assert!(mob.is_avatar);

    mob.attributes.pv.current = 1;
    mob.room_id = ID_ROOM_INIT;

    let player = container.players.find_player_from_avatar_mob_id(mob.id);
    let player = player.unwrap();

    outputs.private(player.id, comm::mob_you_resurrected());
    outputs.room(player.id, mob.room_id, comm::mob_resurrected(mob.label.as_ref()));

    container.mobs.update(mob);
}

pub fn instantiate_from_prefab<'a>(objs: &mut Objects, mobs:  &'a mut MobRepository, items: &mut ItemRepository, mob_prefab_id: MobPrefabId, room_id: RoomId) -> &'a Mob {
    // TODO: mob prefab need to be outside of prefab or manage it inside
    let prefab = mobs.get_mob_prefab(mob_prefab_id).clone();

    // create mob
    let mob_id = objs.insert();

    // add items
    let inventory = prefab.inventory.clone();
    for item_prefab_id in inventory {
        items.instantiate_item(objs, item_prefab_id, mob_id);
    }

    // instantiate
    let mob = Mob::new(mob_id, room_id, prefab.label, prefab.attributes);
    mobs.add(mob);
    mobs.get(mob_id)
}

