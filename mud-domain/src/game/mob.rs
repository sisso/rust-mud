use std::collections::HashMap;

use commons::*;
use logs::*;

use super::container::Container;
use crate::errors::Error::InvalidStateFailure;
use crate::errors::{AsResult, Error, Result, ResultError};
use crate::game::domain::{Attribute, Rd};
use crate::game::inventory_service;
use crate::game::item::ItemPrefabId;
use crate::game::labels::Labels;
use crate::game::location;
use crate::game::location::Locations;
use crate::game::outputs::Outputs;
use crate::game::room::RoomId;
use crate::game::{avatars, combat, comm};
use serde::{Deserialize, Serialize};

pub const EXTRACT_TIME: DeltaTime = DeltaTime(5.0);

pub type MobId = ObjId;
pub type Xp = u32;

/// What mob should be doing
#[derive(Clone, Debug, Copy, Deserialize, Serialize, PartialEq)]
pub enum MobCommand {
    None,
    Kill { target_id: MobId },
    Extract { target_id: ObjId },
}

impl MobCommand {
    // TODO: is idle vs mob.is_idle???
    pub fn is_idle(&self) -> bool {
        match self {
            MobCommand::None => true,
            _ => false,
        }
    }
}

/// What is current doing
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MobAction {
    None,
    Combat,
    Resting,
    Extracting,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Damage {
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MobState {
    // after this total time can attack
    pub attack_calm_down: TotalTime,
    // after this total time can heal
    pub heal_calm_down: TotalTime,
    pub extract_calm_down: TotalTime,
    pub action: MobAction,
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_calm_down: TotalTime(0.0),
            heal_calm_down: TotalTime(0.0),
            extract_calm_down: TotalTime(0.0),
            action: MobAction::None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mob {
    pub id: MobId,
    pub is_avatar: bool,
    pub command: MobCommand,
    pub attributes: Attributes,
    pub xp: Xp,
    pub state: MobState,
    pub followers: Vec<ObjId>,
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
            followers: Default::default(),
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

    pub fn is_idle(&self) -> bool {
        self.state.action == MobAction::None
    }

    pub fn is_combat(&self) -> bool {
        self.state.action == MobAction::Combat
    }

    pub fn is_resting(&self) -> bool {
        self.state.action == MobAction::Resting
    }

    pub fn set_action_rest(&mut self, total: TotalTime) -> Result<()> {
        if !self.is_idle() {
            Err(InvalidStateFailure)
        } else {
            self.state.action = MobAction::Resting;
            self.state.heal_calm_down = TimeTrigger::next(self.attributes.pv.heal_rate, total);
            Ok(())
        }
    }

    pub fn set_action_extract(&mut self, target_id: ObjId, total: TotalTime) -> Result<()> {
        if !self.is_idle() {
            Err(InvalidStateFailure)
        } else {
            self.command = MobCommand::Extract { target_id };
            self.state.action = MobAction::Extracting;
            self.state.extract_calm_down = TimeTrigger::next(EXTRACT_TIME, total);
            Ok(())
        }
    }

    pub fn stop_rest(&mut self) -> Result<()> {
        match self.state.action {
            MobAction::Resting => {
                self.state.action = MobAction::None;
                Ok(())
            }
            _ => Err(InvalidStateFailure),
        }
    }

    pub fn set_action_attack(&mut self, target_id: ObjId) -> Result<()> {
        self.command = MobCommand::Kill { target_id };
        self.state.action = MobAction::Combat;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MobRepository {
    index: HashMap<MobId, Mob>,
}

impl MobRepository {
    pub fn new() -> Self {
        MobRepository {
            index: HashMap::new(),
        }
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &'a Mob> + 'a {
        self.index.iter().map(|(_id, mob)| mob)
    }

    pub fn list_ids<'a>(&'a mut self) -> impl Iterator<Item = MobId> + 'a {
        self.index.iter().map(|(id, _mob)| *id)
    }

    pub fn list_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Mob> + 'a {
        self.index.iter_mut().map(|(_id, mob)| mob)
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
            Err(Error::InvalidArgumentFailure)
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

    pub fn get_mut(&mut self, id: MobId) -> Option<&mut Mob> {
        self.index.get_mut(&id)
    }

    pub fn exists(&self, id: MobId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn set_mob_attack_target(&mut self, mob_id: MobId, target: MobId) -> Result<()> {
        let mob = self.index.get_mut(&mob_id).unwrap();
        mob.set_action_attack(target)
    }

    pub fn cancel_command(&mut self, mob_id: MobId) -> Result<()> {
        let mut mob = self.index.get_mut(&mob_id).as_result()?;
        mob.command = MobCommand::None;
        mob.state.action = MobAction::None;
        Ok(())
    }

    pub fn is_avatar(&self, id: MobId) -> bool {
        self.index.get(&id).unwrap().is_avatar
    }

    pub fn add_follower(&mut self, id: MobId, follower_id: MobId) -> Result<()> {
        let mob = self.get_mut(id).as_result_str("mob not found")?;
        mob.followers.push(follower_id);
        Ok(())
    }

    pub fn remove_follower(&mut self, id: MobId, follower_id: MobId) -> Result<()> {
        let mob = self.get_mut(id).as_result_str("mob not found")?;
        mob.followers.retain(|i| *i != follower_id);
        Ok(())
    }
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
pub fn get_attributes_with_bonus(container: &Container, mob_id: MobId) -> Result<Attributes> {
    let mut attributes = container
        .mobs
        .get(mob_id)
        .ok_or(Error::NotFoundFailure)
        .map(|mob| mob.attributes.clone())?;

    container
        .equips
        .get(mob_id)
        .into_iter()
        .map(|item_id| container.items.get(item_id).unwrap())
        .for_each(|item| {
            if let Some(armor) = item.armor.as_ref() {
                attributes.rd += armor.rd;
                attributes.defense = armor.defense.apply(attributes.defense);
            }

            if let Some(weapon) = item.weapon.as_ref() {
                attributes.attack = weapon.attack.apply(attributes.attack);
                attributes.damage.max += weapon.damage.max;
                attributes.damage.min += weapon.damage.min;
                attributes.attack_calm_down = weapon.calm_down;
            }
        });

    Ok(attributes)
}

pub fn system_run(container: &mut Container) {
    let mut attacks = vec![];
    let mut extracts = vec![];

    for mob in container.mobs.list() {
        match mob.command {
            MobCommand::Kill { target_id } => attacks.push((mob.id, target_id)),
            MobCommand::Extract { target_id } => extracts.push((mob.id, target_id)),
            _ => {}
        };
    }

    // execute extracts
    for (mob_id, target_id) in &extracts {
        match super::extractable::tick_extract(container, *mob_id, *target_id) {
            Err(err) => warn!("{:?} fail to execute extract: {:?}", mob_id, err),
            _ => {}
        };
    }

    // execute attacks
    for (mob_id, target_id) in &attacks {
        match super::combat::tick_attack(container, *mob_id, *target_id) {
            Err(err) => warn!("{:?} fail to execute attack: {:?}", mob_id, err),
            _ => {}
        };
    }
}
