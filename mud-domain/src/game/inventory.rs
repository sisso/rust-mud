use crate::game::item::{Item, ItemId, ItemRepository};
use crate::game::labels::Labels;
use crate::game::location;
use crate::game::location::Locations;
use commons::ObjId;

pub fn move_all(locations: &mut Locations, from: ObjId, to: ObjId) {
    let list: Vec<_> = locations.list_at(from).collect();
    for i in list {
        locations.set(i, to);
    }
}

pub fn get_inventory_list<'a>(
    locations: &Locations,
    items: &'a ItemRepository,
    obj_id: ObjId,
) -> Vec<&'a Item> {
    locations
        .list_at(obj_id)
        .flat_map(|id| items.get(id))
        .collect()
}

pub fn search(
    labels: &Labels,
    locations: &Locations,
    items: &ItemRepository,
    location_id: ObjId,
    label: &str,
) -> Vec<ItemId> {
    location::search_at(labels, locations, location_id, label)
        .into_iter()
        .filter(|obj_id| items.exists(*obj_id))
        .collect()
}

pub fn search_one(
    labels: &Labels,
    locations: &Locations,
    items: &ItemRepository,
    location_id: ObjId,
    label: &str,
) -> Option<ItemId> {
    let mut list = search(labels, locations, items, location_id, label);
    // TODO: how to pop first element?
    list.pop()
}
