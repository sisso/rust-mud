use std::collections::HashMap;

use rand::Rng;

use super::comm;
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
    attack_ready: Seconds
}

impl MobState {
    fn new() -> Self {
        MobState {
            attack_ready: Seconds(0.0),
        }
    }

    fn is_read_to_attack(&self) -> bool {
        self.attack_ready.0 <= 0.0
    }

    fn tick(&mut self, delta: &Seconds) {
        if self.attack_ready.0 > 0.0 {
            self.attack_ready = self.attack_ready - *delta;
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
            state: MobState::new()
        }
    }

    pub fn add_attack_calm_time(&mut self) {
        self.state.attack_ready = self.state.attack_ready + self.attributes.damage.calm_down;
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
}

pub fn run(delta: &Seconds, container: &mut Container, outputs: &mut Outputs) {
    for mob_id in container.mobs.list() {
        if !container.mobs.exists(&mob_id) {
            continue;
        }

        let mut mob = container.mobs.get(&mob_id).clone();
        mob.state.tick(delta);
        container.mobs.update(mob);

        let mob = container.mobs.get(&mob_id);
        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                run_kill(container, outputs, &mob_id, &target);
            }
        }
    }
}

fn run_kill(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target_mob_id: &MobId) {
    let attacker = container.mobs.get(&mob_id);
    let defender = container.mobs.find(&target_mob_id);

    if let Some(defender) = defender {
        // TODO: how send references?
        if attacker.room_id != defender.room_id {
            kill_cancel(container, outputs, &mob_id, Some(target_mob_id));
            return;
        }

        if attacker.state.is_read_to_attack() {
            execute_attack(container, outputs, &mob_id, &target_mob_id);
        }
    } else {
        kill_cancel(container, outputs, &mob_id, None);
    }
}

fn kill_cancel(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target: Option<&MobId>) {
//    let attacker = container.get_mob(&mob_id.0);

//    let msg_others = comm::kill_cancel(attacker, defender);
//
//    if attacker.is_avatar {
//        let player = container.find_player_from_avatar_mob_id(&MobId(attacker.id)).unwrap();
//        let msg_player = comm::kill_player_cancel(defender);
//        outputs.private(player.id.clone(), msg_player);
//        outputs.room(player.id.clone(), attacker.room_id,msg_others);
//    } else {
//        outputs.room_all( attacker.room_id, msg_others);
//    }

//    let mut mob = attacker.clone();
    let mut mob = container.mobs.get(&mob_id).clone();
    mob.command = MobCommand::None;
    container.mobs.update(mob);
}

fn execute_attack(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target: &MobId) {
    let player_id = container.players.find_player_id_from_avatar_mob_id(mob_id);

    let attacker = container.mobs.get(&mob_id);
    let defender = container.mobs.get(&target);

    let attack_result = roll_attack(&attacker.attributes.attack, &attacker.attributes.damage, &defender.attributes.defense);
    let room_attack_msg = comm::kill_mob_execute_attack(attacker, defender, &attack_result);

    if let Some(player_id) = player_id {
        let player_attack_msg = comm::kill_player_execute_attack(&defender, &attack_result);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    if attack_result.success {
        // deduct pv
        let mut defender = container.mobs.get(&target).clone();
        defender.attributes.pv.current -= attack_result.damage as i32;
        container.mobs.update(defender);

        let defender = container.mobs.get(&target);
        if defender.attributes.pv.current < 0 {
            run_mob_killed(container, outputs, mob_id, target);
        }
    }

    let mut attacker = container.mobs.get(&mob_id).clone();
    attacker.add_attack_calm_time();
    container.mobs.update(attacker);
}

// TODO: create body
fn run_mob_killed(container: &mut Container, outputs: &mut Outputs, attacker_id: &MobId, target_id: &MobId) {
    let attacker_player_id = container.players.find_player_id_from_avatar_mob_id(attacker_id);
    let attacker = container.mobs.get(&attacker_id);
    let defender = container.mobs.get(&target_id);

    let room_attack_msg = comm::killed(&defender);

    if let Some(player_id) = attacker_player_id {
        let player_attack_msg = comm::killed_by_player(&defender);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    container.mobs.remove(target_id);
}

fn roll_attack(attack: &u32, damage: &Damage, defense: &u32) -> AttackResult {
    let attack_dice = roll_dice() + *attack;
    let defense_dice = roll_dice() + *defense;

    if attack_dice < defense_dice {
        return AttackResult {
            success: false,
            damage: 0,
            attack_dice,
            defense_dice,
        };
    }

    let damage = roll_damage(damage);

    AttackResult {
        success: true,
        damage,
        attack_dice,
        defense_dice,
    }
}

fn roll_dice() -> u32 {
    let mut rng = rand::thread_rng();

    [0..2].iter()
        .map(|_| rng.gen_range(1, 6 + 1))
        .sum()
}

fn roll_damage(damage: &Damage) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(damage.min, damage.max + 1)
}
