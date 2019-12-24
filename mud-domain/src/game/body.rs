use super::comm;
use super::container::*;
use super::item::*;
use super::mob::*;
use super::Outputs;
use crate::game::inventory;
use crate::game::labels::Label;
use commons::DeltaTime;
use logs::*;

const DECAY_TIME: DeltaTime = DeltaTime(20.0);

pub fn create_body(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) {
    let body_id = container.objects.create();
    let room_id = container.locations.get(mob_id).unwrap();
    let mob_label = container.labels.get_label(mob_id).unwrap();

    let mut body = Item::new(body_id);
    body.decay = Some(container.time.total + DECAY_TIME);
    body.flags.is_body = true;
    container.items.add(body);

    container.locations.set(body_id, room_id);

    let body_label = format!("the body of {}", mob_label);

    container.labels.set(Label {
        id: body_id,
        label: body_label.clone(),
        code: "body".to_string(),
        desc: body_label.to_string(),
    });
    inventory::move_all(&mut container.locations, mob_id, body_id);

    info!(
        "{:?} body of {:?} created at {:?}",
        body_id, mob_id, room_id
    );

    let msg = comm::item_body_appears_in_room(body_label.as_str());
    outputs.broadcast(None, room_id, msg);
}
