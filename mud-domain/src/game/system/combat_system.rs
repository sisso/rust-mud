use crate::errors::*;
use crate::game::container::Container;
use crate::game::mob::MobCommand;
use crate::game::triggers::EventKind;
use crate::game::{combat, comm};
use commons::unwrap_or_continue;
use logs::*;

// TODO: we should not have issues when the mob get deleted.
//       is this a know bug? A hope?
pub fn run(container: &mut Container) {
    run_combat(container);
}

pub fn run_combat(container: &mut Container) {
    let mut attacks = vec![];

    for mob in container.mobs.list() {
        match mob.command {
            MobCommand::None => {}
            MobCommand::Kill { target_id } => attacks.push((mob.id, target_id)),
        };
    }

    // execute attacks
    for (mob_id, target_id) in &attacks {
        match combat::tick_attack(container, *mob_id, *target_id) {
            Err(err) => warn!("{:?} fail to execute attack: {:?}", mob_id, err),
            _ => {}
        };
    }
}
