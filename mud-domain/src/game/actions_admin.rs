use crate::game::container::Container;
use crate::game::Outputs;
use crate::game::mob::{MobId, kill_mob};

pub fn kill(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    kill_mob(container, outputs, mob_id);
}
