use crate::errors::*;
use crate::game::container::Container;
use crate::game::outputs::Outputs;
use crate::game::system::item_system::DecaySystem;

pub mod combat_system;
pub mod item_system;
pub mod random_room_generators_system;
pub mod rest_system;
pub mod ship_system;
pub mod spawn_system;

trait System {
    fn tick(&mut self, container: &mut Container) -> Result<()>;
}

pub struct Systems {
    decay_system: DecaySystem,
}

impl Systems {
    pub fn new(_container: &mut Container) -> Self {
        Systems {
            decay_system: DecaySystem::new(),
        }
    }

    pub fn tick(&mut self, container: &mut Container) {
        // trigger all scheduled tasks
        container
            .timer
            .tick(container.time.total, &mut container.triggers);
        // execute jobs
        self.decay_system.tick(container).unwrap();
        spawn_system::run(container);
        combat_system::run(container);
        rest_system::run(container);
        ship_system::tick(container);
        random_room_generators_system::run(container);
        container.triggers.clear();
    }
}
