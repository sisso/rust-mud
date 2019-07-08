use std::collections::HashMap;

use super::container::Container;
use super::controller::Outputs;
use super::domain::NextId;
use super::room::RoomId;
use super::mob::MobId;
use super::domain::*;
use super::comm;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemType(pub u32);

pub const ITEM_TYPE_COIN: ItemType = ItemType(0);
pub const ITEM_TYPE_BODY: ItemType = ItemType(1);

#[derive(Debug,Clone,Copy)]
pub enum ItemLocation {
    Limbo,
    Mob { mob_id: MobId },
    Room { room_id: RoomId },
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub typ: ItemType,
    pub label: String,
    pub decay: Option<Seconds>,
    pub location: ItemLocation,
}

impl Item {
    pub fn new(id: ItemId, typ: ItemType, label: String) -> Self {
        Item {
            id,
            typ,
            label,
            decay: None,
            location: ItemLocation::Limbo
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemDefId(pub u32);

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: ItemDefId,
    pub typ: ItemType,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    list: Vec<ItemId>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
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
    room_inventory: HashMap<RoomId, Inventory>,
    mob_inventory: HashMap<MobId, Inventory>,
    def_index: HashMap<ItemDefId, ItemDef>,
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
            index: HashMap::new(),
            room_inventory: Default::default(),
            mob_inventory: Default::default(),
            def_index: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }

    pub fn get(&self, item_id: &ItemId) -> &Item {
        self.index.get(item_id).unwrap()
    }

    pub fn add_to_room(&mut self, mut item: Item, room_id: RoomId) {
        let inventory = self.get_room_inventory(room_id);
        inventory.add(item.id);
        item.location = ItemLocation::Room { room_id };
        self.add(item);
    }

    pub fn list_at(&self, room_id: &RoomId) -> Vec<&Item> {
        match self.room_inventory.get(room_id) {
            Some(inventoy) => {
                inventoy
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

    fn add(&mut self, item: Item) {
        self.index.insert(item.id, item);
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        let item = self.index.remove(item_id).unwrap();
        match item.location {
            ItemLocation::Room { room_id } => {
                let inventory = self.get_room_inventory(room_id);
                inventory.remove(item_id);
            },
            _ => {}
        }
    }

    pub fn list(&self) -> Vec<&Item> {
        self.index.values().collect()
    }

    fn get_room_inventory(&mut self, room_id: RoomId) -> &mut Inventory {
        self.room_inventory.entry(room_id).or_insert(Inventory::new())
    }
}

pub fn run_tick(time: &GameTime, container: &mut Container, outputs: &mut Outputs) {
    let items_to_remove: Vec<ItemId> = container
        .items
        .list()
        .iter()
        .filter_map(|item| {
            match (item.location, item.decay) {
                (ItemLocation::Room { room_id }, Some(sec)) if sec.le(&time.total) => {
                    let msg = comm::item_body_disappears(item);
                    outputs.room_all(room_id, msg);
                    Some(item.id.clone())
                }
                _ => None
            }
        })
        .collect();

    for id in items_to_remove.iter() {
        container.items.remove(&id);
    };
}
