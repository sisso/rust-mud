use crate::game::domain::*;
use crate::game::controller::Outputs;
use crate::game::comm;
use rand::Rng;

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
    pub current: u32,
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
    pub id: u32,
    pub room_id: u32,
    pub label: String,
    pub is_avatar: bool,
    pub command: MobCommand,
    pub attributes: Attributes,
    state: MobState,
}

impl Mob {
    pub fn new(id: u32, room_id: u32, label: String, attributes: Attributes) -> Self {
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
}

#[derive(Clone, Debug)]
pub struct MobPrefab {
    pub id: MobPrefabId,
    pub label: String,
    pub attributes: Attributes,
}

pub fn run(delta: &Seconds, container: &mut Container, outputs: &mut Outputs) {
    for mob_id in container.get_mobs() {
        let mob = container.get_mob_mut(&mob_id.0);
        mob.state.tick(delta);

        match mob.command {
            MobCommand::None => {},
            MobCommand::Kill { target } => {
                run_kill(container, outputs, mob_id, target.clone());
            }
        }
    }
}

fn run_kill(container: &mut Container, outputs: &mut Outputs, mob_id: MobId, target_mob_id: MobId) {
    let attacker = container.get_mob(&mob_id.0);
    let defender = container.get_mob(&target_mob_id.0);

    // TODO: how send references?
    if attacker.room_id != defender.room_id {
        kill_cancel(container, outputs, &mob_id, &target_mob_id);
        return;
    }

    if attacker.state.is_read_to_attack() {
        execute_attack(container, outputs, &mob_id, &target_mob_id);
    }
}

fn kill_cancel(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target: &MobId) {
    let attacker = container.get_mob(&mob_id.0);
    let defender = container.get_mob(&target.0);

    let msg_others = comm::kill_cancel(attacker, defender);

    if attacker.is_avatar {
        let player = container.find_player_from_avatar_mob_id(&MobId(attacker.id)).unwrap();
        let msg_player = comm::kill_player_cancel(defender);
        outputs.private(player.id.clone(), msg_player);
        outputs.room(player.id.clone(), attacker.room_id,msg_others);
    } else {
        outputs.room_all( attacker.room_id, msg_others);
    }

    let mut mob = attacker.clone();
    mob.command = MobCommand::None;
    container.update_mob(mob);
}

fn execute_attack(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId, target: &MobId) {
    let attacker = container.get_mob(&mob_id.0);
    let defender = container.get_mob(&target.0);

    let attack_result = roll_attack(&attacker.attributes.attack, &attacker.attributes.damage, &defender.attributes.defense);
    let room_attack_msg = comm::kill_mob_execute_attack(attacker, defender, &attack_result);

    if attacker.is_avatar {
        let player_attack_msg = comm::kill_player_execute_attack(&defender, &attack_result);
        let player = container.find_player_from_avatar_mob_id(&MobId(attacker.id)).unwrap();
        outputs.private(player.id.clone(), player_attack_msg);
        outputs.room(player.id.clone(), attacker.room_id, room_attack_msg);
    } else {
        outputs.room_all(attacker.room_id, room_attack_msg);
    }

    if attack_result.success {
        let defender = container.get_mob_mut(&target.0);
        defender.attributes.pv.current -= attack_result.damage;

        if defender.attributes.pv.current < 0 {
            // TODO: implement death
        }
    }
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
