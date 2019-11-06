use super::comm;
use super::item::*;
use super::mob::*;
use super::container::*;
use super::Outputs;
use commons::DeltaTime;
use crate::game::inventory;
use logs::*;

const DECAY_TIME: DeltaTime = DeltaTime(20.0);

pub fn create_body(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<(),()> {
    let body_id = container.objects.insert();
    let room_id = container.locations.get(mob_id)?;

    let mut body = Item::new(
        body_id,
        ITEM_KIND_BODY,
//        format!("{} body", mob.label).to_string(),
        "body".to_string()
    );

    body.decay = Some(container.time.total + DECAY_TIME);

    let msg = comm::item_body_appears_in_room(&body);

    container.locations.set(body.id, room_id);
    container.items.add(body);
    inventory::move_all(&mut container.locations, mob_id, body_id);

    outputs.room_all(room_id, msg);

    info!("{:?} body of {:?} created at {:?}", body_id, mob_id, room_id);

    Ok(())
}
