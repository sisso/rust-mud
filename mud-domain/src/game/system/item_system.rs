use crate::game::system::{SystemCtx, System};
use crate::game::item::ItemId;
use crate::game::comm;
use crate::game::triggers::*;
use crate::errors::*;
use logs::*;
use commons::trigger::Listener;
use commons::ObjId;

pub struct DecaySystem {
    decay_listener: Listener,
}

impl DecaySystem {
    pub fn new(triggers: &mut Triggers) -> Self {
        let listener = triggers.registre(Kind::Decay);

        DecaySystem {
            decay_listener: listener
        }
    }
}

impl System for DecaySystem {
    fn tick<'a>(&mut self, ctx: &mut SystemCtx<'a>) -> Result<()> {
        let to_remove: Vec<ObjId> = ctx.container.triggers.take(self.decay_listener)
            .into_iter()
            .map(|event| {
                match event {
                    Event::Obj { obj_id, .. } => *obj_id,
                    other=> panic!("unexpected event from kind")
                }
            })
            .collect();

        for obj_id in to_remove {
            info!("{:?} removed by decay", obj_id);
            let location_id = ctx.container.locations.get(obj_id);
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
    use crate::game::builder;

    #[test]
    pub fn test_decay() {
        let mut container = Container::new();
        let mut systems = Systems::new(&mut container);

        let room_id = builder::add_room(&mut container, "roomt1");
        let item_id = builder::add_item(&mut container, "item1", room_id);
    }
}

