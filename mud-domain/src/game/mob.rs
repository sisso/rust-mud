use std::collections::HashMap;

use crate::game::corpse::create_corpse;
use commons::*;
use logs::*;

use super::container::Container;
use crate::errors::{Error, Result};
use crate::game::container::Ctx;
use crate::game::item::ItemPrefabId;
use crate::game::labels::Labels;
use crate::game::location;
use crate::game::location::Locations;
use crate::game::room::RoomId;
use crate::game::Outputs;
use crate::game::{avatars, combat, comm};
use crate::game::inventory;
use crate::game::domain::{Rd, Attribute};

pub type MobId = ObjId;
pub type Xp = u32;

/// What mob should be doing
#[derive(Clone, Debug, Copy)]
pub enum MobCommand {
    None,
    Kill { target: MobId },
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
#[derive(Clone, Debug, PartialEq)]
pub enum MobAction {
    None,
    Combat,
    Resting,
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
    pub attack: Attribute,
    pub defense: Attribute,
    pub damage: Damage,
    pub pv: Pv,
    pub attack_calm_down: DeltaTime,
    pub rd: Rd,
}

impl Attributes {
    pub fn new() -> Self {
        Attributes {
            attack: 10,
            defense: 10,
            damage: Damage { min: 1, max: 1 },
            pv: Pv {
                current: 10,
                max: 10,
                heal_rate: DeltaTime(60.0),
            },
            attack_calm_down: DeltaTime(1.0),
            rd: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct MobState {
    // after this total time can attack
    attack_calm_down: TotalTime,
    // after this total time can heal
    heal_calm_down: TotalTime,
    action: MobAction,
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_calm_down: TotalTime(0.0),
            heal_calm_down: TotalTime(0.0),
            action: MobAction::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AttackResult {
    pub success: bool,
    pub damage_total: u32,
    /// how much damage was really caused
    pub damage_deliver: u32,
    pub attack_value: u32,
    pub defense_value: u32,
    pub attack_dice: u32,
    pub defense_dice: u32,
    pub defense_rd: u32,
}

impl AttackResult {
    pub fn new(attack: u32, defense: u32, rd: u32) -> Self {
        AttackResult {
            success: false,
            damage_total: 0,
            damage_deliver: 0,
            attack_value: attack,
            defense_value: defense,
            attack_dice: 0,
            defense_dice: 0,
            defense_rd: rd,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: MobId,
    pub is_avatar: bool,
    pub command: MobCommand,
    pub attributes: Attributes,
    pub xp: Xp,
    state: MobState,
}

impl Mob {
    pub fn new(id: MobId) -> Self {
        Mob {
            id,
            is_avatar: false,
            command: MobCommand::None,
            attributes: Attributes::new(),
            xp: 0,
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
            }
            _ => {}
        }
    }

    pub fn update_resting(&mut self, total: TotalTime) -> bool {
        if !self.attributes.pv.is_damaged() {
            return false;
        }

        match TimeTrigger::check_trigger(
            self.attributes.pv.heal_rate,
            self.state.heal_calm_down,
            total,
        ) {
            Some(next) => {
                self.state.heal_calm_down = next;
                self.attributes.pv.current += 1;
                true
            }
            None => false,
        }
    }
}

pub struct MobRepository {
    index: HashMap<MobId, Mob>,
}

impl MobRepository {
    pub fn new() -> Self {
        MobRepository {
            index: HashMap::new(),
        }
    }

    // TODO: iterator of ref?
    pub fn list(&self) -> Vec<MobId> {
        self.index
            .iter()
            .into_iter()
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn add(&mut self, mob: Mob) -> &Mob {
        if self.exists(mob.id) {
            panic!("mob already exists")
        }

        debug!("{:?} add mob {:?}", mob.id, mob);

        let id = mob.id;
        self.index.insert(id, mob);
        self.index.get(&id).unwrap()
    }

    pub fn update<F>(&mut self, id: MobId, f: F) -> Result<()>
    where
        F: FnOnce(&mut Mob),
    {
        if let Some(mob) = self.index.get_mut(&id) {
            f(mob);
            debug!("{:?} updated", mob);
            Ok(())
        } else {
            Err(Error::IllegalArgument)
        }
    }

    pub fn remove(&mut self, id: MobId) {
        if self.index.remove(&id).is_some() {
            debug!("{:?} mob removed ", id);
        }
    }

    pub fn get(&self, id: MobId) -> Option<&Mob> {
        self.index.get(&id)
    }

    pub fn exists(&self, id: MobId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn set_mob_attack_target(&mut self, mob_id: MobId, target: MobId) {
        let mut mob = self.index.get_mut(&mob_id).unwrap();
        mob.command = MobCommand::Kill {
            target: target.clone(),
        };
        mob.state.action = MobAction::Combat;
    }

    pub fn cancel_attack(&mut self, mob_id: MobId) {
        let mut mob = self.index.get_mut(&mob_id).unwrap();
        mob.command = MobCommand::None;
        mob.state.action = MobAction::None;
    }

    pub fn is_avatar(&self, id: MobId) -> bool {
        self.index.get(&id).unwrap().is_avatar
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
        let mob = match ctx.container.mobs.get(mob_id) {
            Some(mob) => mob,
            _ => continue,
        };

        let is_resting = mob.is_resting();

        match mob.command {
            MobCommand::None => {}
            MobCommand::Kill { target } => {
                let _ = combat::tick_attack(ctx.container, ctx.outputs, mob_id, target);
            }
        }

        if is_resting {
            let total_time = ctx.container.time.total;
            let player_id = ctx.container.players.find_from_mob(mob_id).unwrap();

            let mobs = &mut ctx.container.mobs;
            let outputs = &mut ctx.outputs;

            let _ = mobs.update(mob_id, |mob| {
                if mob.update_resting(total_time) {
                    if mob.attributes.pv.is_damaged() {
                        let msg = comm::rest_healing(mob.attributes.pv.current);
                        outputs.private(mob_id, msg);
                    } else {
                        outputs.private(mob_id, comm::rest_healed());
                    }
                }
            });
        }
    }
}

// TODO: move game rules with output outside of mobs module
// TODO: become a trigger?
pub fn kill_mob(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    info!("{:?} was killed", mob_id);

    let _ = create_corpse(container, outputs, mob_id);

    // remove mob
    let mob = container.mobs.get(mob_id).unwrap();
    if mob.is_avatar {
        avatars::respawn_avatar(container, outputs, mob_id)?;
    } else {
        container.remove(mob_id);
    }

    Ok(())
}

pub fn search_mobs_at(
    labels: &Labels,
    locations: &Locations,
    mobs: &MobRepository,
    room_id: RoomId,
    label: &str,
) -> Vec<MobId> {
    location::search_at(labels, locations, room_id, label)
        .into_iter()
        .filter(|&mob_id| mobs.exists(mob_id))
        .collect()
}

/// get mob attributes summing items
pub fn get_attributes(container: &Container, mob_id: MobId) -> Result<Attributes> {
    let mut attributes= container.mobs.get(mob_id)
        .ok_or(Error::NotFound)
        .map(|mob| mob.attributes.clone())?;

    let equipped_items = container.equips.get(mob_id)
        .into_iter()
        .map(|item_id| container.items.get(item_id).unwrap())
        .for_each(|item| {
            if let Some(armor) = item.armor.as_ref() {
                attributes.rd += armor.rd;
                attributes.defense = armor.defense.apply(attributes.defense);
            }

            if let Some(weapon) = item.weapon.as_ref() {
                attributes.attack = weapon.attack.apply(attributes.attack);
                attributes.damage = weapon.damage.clone();
                attributes.attack_calm_down = weapon.calm_down;
            }
        });

    Ok(attributes)
}
