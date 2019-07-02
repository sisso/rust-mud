use rand::Rng;

use super::domain::*;
use super::controller::Outputs;
use super::comm;
use super::room::RoomId;
use super::container::Container;

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

pub fn run(delta: &Seconds, container: &mut Container, outputs: &mut Outputs) {
    for mob_id in container.get_mobs() {
        if !container.is_mob(&mob_id) {
            continue;
        }

        let mob = container.get_mob_mut(&mob_id);
        mob.state.tick(delta);

        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                run_kill(container, outputs, &mob_id.clone(), &target.clone());
            }
        }
    }
}

fn run_kill(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target_mob_id: &MobId) {
    let attacker = container.get_mob(&mob_id);
    let defender = container.find_mob(&target_mob_id);

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
    let mob = container.get_mob_mut(&mob_id);
    mob.command = MobCommand::None;
}

fn execute_attack(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target: &MobId) {
    let player_id = container.players.find_player_id_from_avatar_mob_id(mob_id);

    let attacker = container.get_mob(&mob_id);
    let defender = container.get_mob(&target);

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
        let defender = container.get_mob_mut(&target);
        defender.attributes.pv.current -= attack_result.damage as i32;

        if defender.attributes.pv.current < 0 {
            run_mob_killed(container, outputs, mob_id, target);
        }
    }

    let attacker = container.get_mob_mut(&mob_id);
    attacker.add_attack_calm_time();
}

// TODO: create body
fn run_mob_killed(container: &mut Container, outputs: &mut Outputs, attacker_id: &MobId, target_id: &MobId) {
    let attacker_player_id = container.players.find_player_id_from_avatar_mob_id(attacker_id);
    let attacker = container.get_mob(&attacker_id);
    let defender = container.get_mob(&target_id);

    let room_attack_msg = comm::killed(&defender);

    if let Some(player_id) = attacker_player_id {
        let player_attack_msg = comm::killed_by_player(&defender);
        outputs.private(player_id, player_attack_msg);
        outputs.room(player_id, attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    container.remove_mob(target_id);
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
