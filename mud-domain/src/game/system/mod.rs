use crate::game::container::Container;
use crate::game::Outputs;
use crate::errors::*;

pub mod spawn_system;
pub mod ship_system;
pub mod combat_system;
pub mod rest_system;
pub mod item_system;

pub struct SystemCtx<'a> {
    pub container: &'a mut Container,
    pub outputs: &'a mut dyn Outputs,
}

// Not useful until some system start to have states
//trait System {
//    fn tick(ctx: &mut SystemCtx) -> Result<Ok>;
//}

pub fn run(ctx: &mut SystemCtx) {
    spawn_system::run(ctx);
    combat_system::run(ctx);
    rest_system::run(ctx);
    item_system::run(ctx);
    ship_system::tick(ctx);
}

