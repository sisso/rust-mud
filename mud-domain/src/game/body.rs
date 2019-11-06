use super::comm;
use super::item::*;
use super::mob::*;
use super::container::*;
use super::domain::*;
use super::Outputs;
use commons::DeltaTime;
use crate::game::inventory;

const DECAY_TIME: DeltaTime = DeltaTime(20.0);

pub fn create_body(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<(),()> {
    let item_id = container.objects.insert();
    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob.id)?;

    let mut item = Item::new(
        item_id,
        ITEM_KIND_BODY,
//        format!("{} body", mob.label).to_string(),
        "body".to_string()
    );

    item.decay = Some(container.time.total + DECAY_TIME);

    let msg = comm::item_body_appears_in_room(&item);

    container.locations.set(item.id, room_id);
    container.items.add(item);
    inventory::move_all(&mut container.locations, mob_id, item_id);

    outputs.room_all(room_id, msg);

    Ok(())
}
