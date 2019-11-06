use commons::ObjId;
use crate::game::location::Locations;
use crate::game::item::{Item, ItemRepository};

pub fn move_all(locations: &Locations, from: ObjId, to: ObjId) {
   unimplemented!()
}

pub fn get_inventory_list<'a>(locations: &Locations, items: &'a ItemRepository, obj_id: ObjId) -> Vec<&'a Item> {
   unimplemented!()
}

pub fn search<'a>(locations: &Locations, items: &'a ItemRepository, location_id: ObjId, label: &str) -> Vec<&'a Item> {
   unimplemented!()
}

