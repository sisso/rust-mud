use crate::errors::*;
use crate::game::container::Container;
use crate::game::outputs::Outputs;
use crate::game::system::item_system::DecaySystem;

pub mod ai_system;
pub mod avatars_systems;
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
    pub fn new(container: &mut Container) -> Self {
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
        ai_system::run(container);
        super::mob::system_run(container);
        rest_system::run(container);
        ship_system::tick(container);
        random_room_generators_system::run(container);
        avatars_systems::run(container);
        container.triggers.clear();
    }
}
