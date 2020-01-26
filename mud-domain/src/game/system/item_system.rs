use crate::game::system::{SystemCtx, System};
use crate::game::item::ItemId;
use crate::game::comm;
use crate::game::triggers::*;
use crate::errors::*;
use logs::*;
use commons::ObjId;

pub struct DecaySystem {
}

impl DecaySystem {
    pub fn new() -> Self {
        DecaySystem {
        }
    }
}

impl System for DecaySystem {
    fn tick<'a>(&mut self, ctx: &mut SystemCtx<'a>) -> Result<()> {
        let to_remove: Vec<ObjId> = ctx.container.triggers.list(EventKind::Decay)
            .map(|event| {
                match event {
                    Event::Obj { obj_id, .. } => *obj_id,
                    _other=> panic!("unexpected event from kind")
                }
            })
            .collect();

        for obj_id in to_remove {
            info!("{:?} removed by decay", obj_id);
            if let Some(location_id) = ctx.container.locations.get(obj_id) {
                let label = ctx.container.labels.get_label_f(obj_id);
                let msg = comm::item_body_disappears(label);
                ctx.outputs.broadcast(None, location_id, msg);
            }

            ctx.container.remove(obj_id);
            ctx.container.items.remove(obj_id);
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
    use commons::{TotalTime, DeltaTime};
    use crate::controller::OutputsBuffer;

    #[test]
    pub fn test_decay() {
        let mut scenery = crate::game::test::scenery();

        let room_id = builder::add_room(&mut scenery.container, "room1");
        let item_id = builder::add_item(&mut scenery.container, "item1", room_id);

        scenery.container.timer.schedule(TotalTime(1.0), Event::Obj { kind: EventKind::Decay, obj_id: item_id });

        scenery.tick(0.1);
        assert!(scenery.container.objects.exists(item_id));

        scenery.tick(0.91);
        assert!(!scenery.container.objects.exists(item_id));
    }
}

