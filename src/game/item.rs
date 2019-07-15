use std::collections::HashMap;

use super::container::Container;
use super::controller::Outputs;
use super::domain::NextId;
use super::room::RoomId;
use super::mob::MobId;
use super::domain::*;
use super::comm;

use crate::lib::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemType(pub u32);

pub const ITEM_TYPE_GOLD: ItemType = ItemType(0);
pub const ITEM_TYPE_BODY: ItemType = ItemType(1);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ItemLocation {
    Limbo,
    Mob { mob_id: MobId },
    Room { room_id: RoomId },
    Item { item_id: ItemId },
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub typ: ItemType,
    pub label: String,
    pub decay: Option<Seconds>,
    pub amount: u32,
    pub item_def_id: Option<ItemPrefabId>
}

impl Item {
    pub fn new(id: ItemId, typ: ItemType, label: String) -> Self {
        Item {
            id,
            typ,
            label,
            decay: None,
            amount: 1,
            item_def_id: None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemPrefabId(pub u32);

#[derive(Debug, Clone)]
pub struct ItemPrefab {
    pub id: ItemPrefabId,
    pub typ: ItemType,
    pub label: String,
    pub amount: u32
}

#[derive(Debug, Clone)]
pub struct Inventory {
    location: ItemLocation,
    list: Vec<ItemId>,
}

impl Inventory {
    pub fn new(location: ItemLocation) -> Self {
        Inventory {
            location: location,
            list: vec![]
        }
    }

    pub fn add(&mut self, item_id: ItemId) {
        self.list.push(item_id);
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        self.list.retain(|id| id != item_id);
    }
}

pub struct ItemRepository {
    next_item_id: NextId,
    next_item_def_id: NextId,
    index: HashMap<ItemId, Item>,
    inventory: HashMap<ItemLocation, Inventory>,
    prefab_index: HashMap<ItemPrefabId, ItemPrefab>,
    item_location: HashMap<ItemId, ItemLocation>
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
            index: HashMap::new(),
            inventory: HashMap::new(),
            prefab_index: HashMap::new(),
            item_location: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }

    pub fn get(&self, item_id: &ItemId) -> &Item {
        self.index.get(item_id).unwrap()
    }

    pub fn set_location(&mut self, item_id: ItemId, location: ItemLocation) {
        let inventory = self.get_inventory_mut(location);
        inventory.add(item_id);

        self.item_location.insert(item_id, location);
    }

    pub fn move_all(&mut self, from: ItemLocation, to: ItemLocation) {
        println!("itemrepostitory - move_all {:?} to {:?}", from, to);
        let from_inventory = self.inventory.remove(&from);
        if from_inventory.is_none() {
            return;
        }

        let from_inventory = from_inventory.unwrap();
        for item_id in from_inventory.list.iter() {
            println!("itemrepostitory - set {:?} to {:?}", item_id, to);
            self.item_location.insert(*item_id, to);
        }

        let to_inventory = self.get_inventory_mut(to);
        for item_id in from_inventory.list {
            println!("itemrepostitory - add {:?} to {:?}", item_id, to_inventory);
            to_inventory.add(item_id);
        }
    }

    pub fn move_item(&mut self, item_id: ItemId, location: ItemLocation) {
        self.remove_location(&item_id);
        self.add_location(&item_id, location);
    }

    pub fn move_to_mob(&mut self, mob_id: &MobId, item_id: &ItemId) {
        self.move_item(*item_id, ItemLocation::Mob { mob_id: *mob_id });
    }

    pub fn move_items_from_mob_to_item(&mut self, mob_id: MobId, item_id: ItemId) {
        self.move_all(ItemLocation::Mob { mob_id: mob_id }, ItemLocation::Item { item_id: item_id });
    }

    pub fn get_item_inventory_list(&self, item_id: &ItemId) -> Vec<&Item> {
        self.get_inventory_list(&ItemLocation::Item { item_id: *item_id })
    }

    pub fn get_inventory_list(&self, location: &ItemLocation) -> Vec<&Item> {
        match self.inventory.get(location) {
            Some(inventory) => {
                inventory
                    .list
                    .iter()
                    .map(|item_id| {
                        self.get(&item_id)
                    })
                    .collect()
            },
            None => vec![],
        }
    }

    pub fn add(&mut self, item: Item, location: ItemLocation) {
        let item_id = item.id;

        if self.index.contains_key(&item_id) {
            panic!()
        }

        // update index
        self.index.insert(item_id, item);

        // update inventory
        let inventory = self.get_inventory_mut(location);
        inventory.add(item_id);

        // update location
        self.item_location.insert(item_id, location);
    }

    pub fn add_prefab(&mut self, item_def: ItemPrefab) {
        self.prefab_index.insert(item_def.id, item_def);
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        let item = self.index.remove(item_id).unwrap();
        let location = self.item_location.remove(item_id);
        if let Some(location) = location {
            let inventory = self.get_inventory_mut(location);
            inventory.remove(&item_id);
        }

        // TODO: remove recursive all items in inventory
    }

    pub fn list(&self) -> Vec<&Item> {
        self.index.values().collect()
    }

    pub fn get_prefab(&self, item_prefab_id: &ItemPrefabId) -> &ItemPrefab {
        self.prefab_index.get(item_prefab_id).unwrap()
    }

    pub fn search(&self, room_id: &RoomId, name: &String) -> Vec<&Item> {
        match self.inventory.get(&ItemLocation::Room { room_id: *room_id }) {
            Some(inventory) => {
                inventory.list.iter().filter_map(|item_id| {
                    let item = self.get(item_id);
                    if item.label.eq_ignore_ascii_case(name) {
                        Some(item)
                    } else {
                        None
                    }
                }).collect()
            },
            _ => {
                vec![]
            }
        }
    }

    pub fn get_location(&self, item_id: &ItemId) -> &ItemLocation {
        self.item_location.get(item_id).unwrap()
    }

    pub fn get_inventory(&self, location: &ItemLocation) -> Option<&Inventory> {
        self.inventory.get(location)
    }

    fn get_inventory_mut(&mut self, location: ItemLocation) -> &mut Inventory {
        self.inventory.entry(location).or_insert(Inventory::new(location.clone()))
    }

    fn remove_location(&mut self, item_id: &ItemId) {
        let location = self.get_location(&item_id).clone();
        let inventory = self.get_inventory_mut(location);
        inventory.remove(&item_id);

        self.item_location.remove(item_id);

        println!("itemrepostitory - remove_location {:?}", item_id);
    }

    fn add_location(&mut self, item_id: &ItemId, location: ItemLocation) {
        let inventory = self.get_inventory_mut(location);
        inventory.add(*item_id);

        self.item_location.insert(*item_id, location);

        println!("itemrepostitory - add_location {:?} {:?}", item_id, location);

        let inventory = self.get_inventory(&location);
        println!("itemrepostitory - inventory {:?}", inventory);
    }
}

pub fn run_tick(time: &GameTime, container: &mut Container, outputs: &mut Outputs) {
    let items_to_remove: Vec<ItemId> = container
        .items
        .list()
        .iter()
        .filter_map(|item| {
            if let Some(decay) = item.decay {
                let location = container.items.get_location(&item.id);

                match location {
                    ItemLocation::Room { room_id } if decay.le(&time.total) => {
                        let msg = comm::item_body_disappears(item);
                        outputs.room_all(*room_id, msg);
                        Some(item.id.clone())
                    }
                    _ => None
                }
            } else {
                None
            }

        })
        .collect();

    for id in items_to_remove.iter() {
        container.items.remove(&id);
    };
}
