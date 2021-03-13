use crate::errors::*;
use crate::game::comm;
use crate::game::container::Container;
use crate::game::item::ItemId;
use crate::game::system::System;
use crate::game::triggers::*;
use commons::ObjId;
use logs::*;

pub struct DecaySystem {}

impl DecaySystem {
    pub fn new() -> Self {
        DecaySystem {}
    }
}

impl System for DecaySystem {
    fn tick(&mut self, container: &mut Container) -> Result<()> {
        let to_remove: Vec<ObjId> = container
            .triggers
            .list(EventKind::Decay)
            .map(|event| match event {
                Event::Obj { obj_id, .. } => *obj_id,
            })
            .collect();

        for obj_id in to_remove {
            info!("{:?} removed by decay", obj_id);
            if let Some(location_id) = container.locations.get(obj_id) {
                let label = container.labels.get_label_f(obj_id);
                let msg = comm::item_body_disappears(label);
                container.outputs.broadcast(None, location_id, msg);
            }

            container.remove(obj_id);
            container.items.remove(obj_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::container::Container;
    use crate::game::system::Systems;
    use crate::game::{builder, main_loop};
    use commons::{DeltaTime, TotalTime};

    #[test]
    pub fn test_decay() {
        let mut scenery = crate::game::test::scenery();

        let room_id = builder::add_room(&mut scenery.container, "room1");
        let item_id = builder::add_item(&mut scenery.container, "item1", room_id);

        scenery.container.timer.schedule(
            TotalTime(1.0),
            Event::Obj {
                kind: EventKind::Decay,
                obj_id: item_id,
            },
        );

        scenery.tick(0.1);
        assert!(scenery.container.objects.exists(item_id));

        scenery.tick(0.91);
        assert!(!scenery.container.objects.exists(item_id));
    }
}
