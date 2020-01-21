use crate::game::container::Container;
use crate::game::Outputs;
use crate::errors::*;
use crate::game::system::item_system::DecaySystem;

pub mod spawn_system;
pub mod ship_system;
pub mod combat_system;
pub mod rest_system;
pub mod item_system;

// TODO: since contains only mutable references, we can send it by value
pub struct SystemCtx<'a> {
    pub container: &'a mut Container,
    pub outputs: &'a mut dyn Outputs,
}

trait System {
    fn tick(&mut self, ctx: &mut SystemCtx) -> Result<()>;
}

pub struct Systems {
    decay_system: DecaySystem,
}

impl Systems {
    pub fn new(container: &mut Container) -> Self {
        Systems {
            decay_system: DecaySystem::new(),
        }
    }

    pub fn tick(&mut self, ctx: &mut SystemCtx) {
        // trigger all scheduled tasks
        ctx.container.timer.tick(ctx.container.time.total, &mut ctx.container.triggers);
        // execute jobs
        self.decay_system.tick(ctx).unwrap();
        spawn_system::run(ctx);
        combat_system::run(ctx);
        rest_system::run(ctx);
        ship_system::tick(ctx);
        ctx.container.triggers.clear();
    }
}

