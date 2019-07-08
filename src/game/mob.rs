use std::collections::HashMap;

use super::combat;
use super::container::Container;
use super::controller::Outputs;
use super::domain::*;
use super::room::RoomId;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobId(pub u32);

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobPrefabId(pub u32);

#[derive(Clone, Debug)]
pub enum MobCommand {
    None,
    Kill { target: MobId }
}

#[derive(Clone, Debug)]
pub struct Damage {
    pub min: u32,
    pub max: u32,
    pub calm_down: Seconds,
}

#[derive(Clone, Debug)]
pub struct Pv {
    pub current: i32,
    pub max: u32,
}

#[derive(Clone, Debug)]
pub struct Attributes {
    pub attack: u32,
    pub defense: u32,
    pub damage: Damage,
    pub pv: Pv,
}

#[derive(Clone, Debug)]
struct MobState {
    // change to ready on > current time
    attack_ready: Seconds
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_ready: Seconds(0.0),
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

    pub fn add_attack_calm_time(&mut self, total_time: &Seconds) {
        self.state.attack_ready = *total_time + self.attributes.damage.calm_down;
    }

    pub fn is_read_to_attack(&self, total_time: &Seconds) -> bool {
        self.state.attack_ready.0 <= total_time.0
    }
}

#[derive(Clone, Debug)]
pub struct MobPrefab {
    pub id: MobPrefabId,
    pub label: String,
    pub attributes: Attributes,
}

pub struct MobRepository {
    next_id: NextId,
    index: HashMap<MobId, Mob>
}

impl MobRepository {
    pub fn new() -> Self {
        MobRepository {
            next_id: NextId::new(),
            index: HashMap::new()
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
        if !self.exists(&mob.id) {
            panic!("mob do not exists")
        }
        let id = mob.id;
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

    pub fn set_mob_kill_target(&mut self, id: &MobId, target: &MobId) {
        let mut mob = self.index.get_mut(id).unwrap();
        mob.command = MobCommand::Kill { target: target.clone() };
    }

    pub fn is_avatar(&self, id: &MobId) -> bool {
        self.index.get(id).unwrap().is_avatar
    }
}

pub fn run_tick(time: &GameTime, container: &mut Container, outputs: &mut Outputs) {
    for mob_id in container.mobs.list() {
        if !container.mobs.exists(&mob_id) {
            continue;
        }

        let mob = container.mobs.get(&mob_id);
        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                combat::run_attack(time, container, outputs, &mob_id, &target);
            }
        }
    }
}
