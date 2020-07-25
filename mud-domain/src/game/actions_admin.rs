use crate::errors::Result;
use crate::game::container::Container;
use crate::game::mob::{kill_mob, MobId};
use crate::game::outputs::Outputs;

pub fn force_kill(container: &mut Container, mob_id: MobId) -> Result<()> {
    kill_mob(container, mob_id)
}
