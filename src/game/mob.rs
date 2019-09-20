use std::collections::HashMap;

use crate::game::body::create_body;
use crate::game::Ctx;
use crate::utils::*;
use crate::utils::save::Save;

use super::combat;
use super::comm;
use super::container::Container;
use super::domain::*;
use super::item::*;
use super::Outputs;
use super::room::RoomId;

pub const INITIAL_ROOM_ID: RoomId = RoomId(0);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobId(pub u32);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobPrefabId(pub u32);

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
    pub heal_rate: Second,
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
    pub attack_calm_down: Second,
}

#[derive(Clone, Debug)]
struct MobState {
    // after this total time can attack
    attack_calm_down: Second,
    // after this total time can heal
    heal_calm_down: Second,
    action: MobAction
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_calm_down: Second(0.0),
            heal_calm_down: Second(0.0),
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

    pub fn add_attack_calm_time(&mut self, total_time: Second) {
        let next = TimeTrigger::next(self.attributes.attack_calm_down, total_time);
        self.state.attack_calm_down = next;
    }

    pub fn is_read_to_attack(&self, total_time: Second) -> bool {
        let trigger = TimeTrigger::should_trigger(self.state.attack_calm_down, total_time);
        trigger
    }

    pub fn is_combat(&self) -> bool {
        self.state.action == MobAction::Combat
    }

    pub fn is_resting(&self) -> bool {
        self.state.action == MobAction::Resting
    }

    pub fn set_action(&mut self, action: MobAction, total: Second) {
        self.state.action = action;

        match self.state.action {
            MobAction::Resting => {
                self.state.heal_calm_down = TimeTrigger::next(self.attributes.pv.heal_rate, total);
            },
            _ => {}
        }
    }

    pub fn update_resting(&mut self, total: Second) -> bool {
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
    next_id: NextId,
    index: HashMap<MobId, Mob>,
    mob_prefabs: HashMap<MobPrefabId, MobPrefab>,
}

impl MobRepository {
    pub fn new() -> Self {
        MobRepository {
            next_id: NextId::new(),
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

    pub fn new_id(&mut self) -> MobId {
        let id = self.next_id.next();
        MobId(id)
    }

    pub fn add(&mut self, mob: Mob) -> &Mob {
        if self.exists(&mob.id) {
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

    pub fn get(&self, id: &MobId) -> &Mob {
        self.index.get(id).unwrap()
    }

    pub fn find(&self, id: &MobId) -> Option<&Mob> {
        self.index.get(id)
    }

    pub fn exists(&self, id: &MobId) -> bool {
        self.index.contains_key(id)
    }

    pub fn search(&self, room_id: Option<&RoomId>, name: Option<&String>) -> Vec<&Mob> {
        self.index
            .iter()
            .filter(|(_, mob)| {
                if let Some(room_id) = room_id {
                    if mob.room_id != *room_id {
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

    pub fn set_mob_attack_target(&mut self, mob_id: MobId, target: &MobId) {
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
        self.mob_prefabs.insert(mob_prefab.id, mob_prefab);
    }

    pub fn get_mob_prefab(&mut self, id: &MobPrefabId) -> &MobPrefab {
        self.mob_prefabs.get(id)
            .expect(format!("could not found mob prefab id {:?}", id).as_str())
    }

    pub fn save(&self, save: &mut dyn Save) {
        use serde_json::json;

        for (id, obj) in self.index.iter() {
            let command_json = match obj.command {
                MobCommand::None => json!({ "kind": "idle" }),
                MobCommand::Kill { target } => json!({ "kind": "kill", "target": target.0 }),
            };

            let obj_json = json!({
                "room_id": obj.room_id.0,
                "label": obj.label,
                "is_avatar": obj.is_avatar,
                "command": command_json,
                "attributes": {
                    "attack": obj.attributes.attack,
                    "defense": obj.attributes.defense,
                    "damage_min": obj.attributes.damage.min,
                    "damage_max": obj.attributes.damage.max,
                    "damage_calm_down": obj.attributes.attack_calm_down.as_float(),
                    "pv": obj.attributes.pv.current,
                    "pv_max": obj.attributes.pv.max,
                    "pv_heal_rate": obj.attributes.pv.heal_rate.as_float(),
                },
                "state": {
                    "attack_ready": obj.state.attack_calm_down.as_float(),
                    "heal_ready": obj.state.heal_calm_down.as_float(),
                    "action": match obj.state.action {
                        MobAction::None => "none",
                        MobAction::Combat => "combat",
                        MobAction::Resting => "rest",
                    },
                }
            });

            save.add(id.0, "mob", obj_json);
        }
    }
}

pub fn run_tick(ctx: &mut Ctx) {
    for mob_id in ctx.container.mobs.list() {
        if !ctx.container.mobs.exists(&mob_id) {
            continue;
        }

        let mob = ctx.container.mobs.get(&mob_id);

        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                combat::tick_attack(ctx.time, ctx.container, ctx.outputs, mob_id, target);
            }
        }

        let mob = ctx.container.mobs.get(&mob_id);
        if mob.is_resting() {
            let mut mob = mob.clone();
            if mob.update_resting(ctx.time.total) {
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

pub fn kill_mob(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    create_body(time, container, outputs, mob_id);

    // remove mob
    let mob = container.mobs.get(&mob_id);
    if mob.is_avatar {
        respawn_avatar(time, container, outputs, mob_id);
    } else {
        container.mobs.remove(&mob_id);
    }
}

pub fn respawn_avatar(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    let mut mob = container.mobs.get(&mob_id).clone();
    assert!(mob.is_avatar);

    mob.attributes.pv.current = 1;
    mob.room_id = INITIAL_ROOM_ID;

    let player = container.players.find_player_from_avatar_mob_id(mob.id);
    let player = player.unwrap();

    outputs.private(player.id, comm::mob_you_resurrected());
    outputs.room(player.id, mob.room_id, comm::mob_resurrected(mob.label.as_ref()));

    container.mobs.update(mob);
}
