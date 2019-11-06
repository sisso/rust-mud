use commons::ObjId;
use crate::game::location::Locations;
use crate::game::item::{Item, ItemRepository};

pub fn move_all(locations: &mut Locations, from: ObjId, to: ObjId) {
   let list: Vec<_> = locations.list_at(from).collect();
   for i in list {
      locations.set(i, to);
   }
}

pub fn get_inventory_list<'a>(locations: &Locations, items: &'a ItemRepository, obj_id: ObjId) -> Vec<&'a Item> {
   locations.list_at(obj_id).flat_map(|id| {
      items.get(id).ok()
   }).collect()
}

pub fn search<'a>(locations: &Locations, items: &'a ItemRepository, location_id: ObjId, label: &str) -> Vec<&'a Item> {
   locations.list_at(location_id).flat_map(|id| {
      match items.get(id) {
         Ok(item) if item.label.as_str().eq_ignore_ascii_case(label) => Some(item),
         _ => None
      }
   }).collect()}

pub fn search_one<'a>(locations: &Locations, items: &'a ItemRepository, location_id: ObjId, label: &str) -> Option<&'a Item> {
   let mut list = search(locations, items, location_id, label);
   // TODO: how to pop first element?
   list.pop()
}

