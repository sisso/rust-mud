//static mut NEXT_MOB_ID: u32 = 0;

use crate::game::domain::*;
use crate::game::controller::Outputs;
use crate::game::comm;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobId(pub u32);

//impl MobId {
//    unsafe fn next_id() -> MobId  {
//        let next = NEXT_MOB_ID;
//        NEXT_MOB_ID += 1;
//        MobId(next)
//    }
//}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct MobPrefabId(pub u32);

#[derive(Clone, Debug)]
pub enum MobCommand {
    None,
    Kill { target: MobId }
}

#[derive(Clone, Debug)]
pub struct Mob {
    pub id: u32,
    pub room_id: u32,
    pub label: String,
    pub is_avatar: bool,
    pub command: MobCommand,
}

impl Mob {
    pub fn new(id: u32, room_id: u32, label: String) -> Self {
        Mob {
            id,
            room_id,
            label,
            is_avatar: false,
            command: MobCommand::None
        }
    }
}

pub fn run(container: &mut Container, outputs: &mut Outputs) {
    for mob_id in container.get_mobs() {
        let mob = container.get_mob(&mob_id.0);

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
