use commons::DeltaTime;
use crate::game::container::Container;
use crate::game::system::{Systems, SystemCtx};
use crate::game::Outputs;
use crate::game::system;
use logs::*;

pub fn tick(delta_time: DeltaTime, container: &mut Container, systems: &mut Systems, outputs: &mut dyn Outputs) {
    container.time.add(delta_time);

    if container.time.tick.as_u32() % 100 == 0 {
        debug!("tick {:?}", container.time);
    }

    let mut ctx = SystemCtx { container, outputs, };

    // TODO: inputs
    systems.tick(&mut ctx);
    // TODO: after rum? trigger?
    // TODO: outputs
    container.triggers.clear();
}
