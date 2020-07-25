use crate::game::container::Container;
use crate::game::outputs::Outputs;
use crate::game::system::Systems;
use crate::game::{system, GameCfg};
use commons::DeltaTime;
use logs::*;

pub fn tick(delta_time: DeltaTime, container: &mut Container, systems: &mut Systems) {
    container.time.add(delta_time);

    if container.time.tick.as_u32() % 100 == 0 {
        debug!("tick {:?}", container.time);
    }

    // TODO: inputs
    systems.tick(container);
    // TODO: after rum? trigger?
    // TODO: outputs
    container.triggers.clear();
}
