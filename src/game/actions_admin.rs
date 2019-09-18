use crate::game::domain::GameTime;
use crate::game::container::Container;
use crate::game::runner::Outputs;
use crate::game::mob::{MobId, kill_mob};

pub fn kill(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    kill_mob(time, container, outputs, mob_id);
}
