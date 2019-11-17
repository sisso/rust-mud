use crate::game::container::Container;
use crate::game::Outputs;
use crate::game::mob::{MobId, kill_mob};
use commons::UResult;

pub fn kill(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> UResult {
    kill_mob(container, outputs, mob_id)
}
