use crate::game::container::Container;
use crate::game::mob::{kill_mob, MobId};
use crate::game::Outputs;
use crate::errors::Result;

pub fn kill(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    kill_mob(container, outputs, mob_id)
}
