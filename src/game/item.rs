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

pub const ITEM_TYPE_GOLD: ItemType = ItemType(0);
pub const ITEM_TYPE_BODY: ItemType = ItemType(1);

#[derive(Debug,Clone,Copy)]
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
    pub item_def_id: Option<ItemDefId>
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
pub struct ItemDefId(pub u32);

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: ItemDefId,
    pub typ: ItemType,
    pub label: String,
    pub amount: u32
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
    item_inventory: HashMap<ItemId, Inventory>,
    def_index: HashMap<ItemDefId, ItemDef>,
    item_location: HashMap<ItemId, ItemLocation>
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
            index: HashMap::new(),
            room_inventory: Default::default(),
            mob_inventory: Default::default(),
            item_inventory: Default::default(),
            def_index: HashMap::new(),
            item_location: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }

    pub fn get(&self, item_id: &ItemId) -> &Item {
        self.index.get(item_id).unwrap()
    }

    pub fn add_to_room(&mut self, mut item: Item, room_id: RoomId) {
        let inventory = self.get_room_inventory(&room_id);
        inventory.add(item.id);

        self.item_location.insert(item.id, ItemLocation::Room { room_id });
        self.index.insert(item.id, item);
    }

    pub fn add_to_mob(&mut self, item: Item, mob_id: MobId) {
        let inventory = self.get_mob_inventory(&mob_id);
        inventory.add(item.id);
        self.item_location.insert(item.id, ItemLocation::Mob { mob_id });
        self.index.insert(item.id, item);
    }

    pub fn move_items_from_mob_to_item(&mut self, mob_id: MobId, item_id: ItemId) {
        if let Some(mob_inventory) = self.mob_inventory.remove(&mob_id) {
            // add all items to item inventory
            let item_inventory = self.get_item_inventory(&item_id);
            for local_item_id in mob_inventory.list.iter() {
                item_inventory.add(*local_item_id);
            }

            // move item to new location
            for local_item_id in mob_inventory.list {
                self.item_location.insert(local_item_id, ItemLocation::Item { item_id });
            }
        }
    }

    pub fn list_at(&self, room_id: &RoomId) -> Vec<&Item> {
        match self.room_inventory.get(room_id) {
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

    pub fn add(&mut self, item: Item) {
        self.index.insert(item.id, item);
    }

    pub fn add_def(&mut self, item_def: ItemDef) {
        self.def_index.insert(item_def.id, item_def);
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        let item = self.index.remove(item_id).unwrap();
        // TODO: remove clone
        let location = self.item_location.get(item_id).unwrap().clone();
        let mut inventory = self.get_location_inventory_mut(&location);
        inventory.iter_mut().for_each(|inventory| {
            inventory.remove(&item_id);
        });
    }

    pub fn list(&self) -> Vec<&Item> {
        self.index.values().collect()
    }


    pub fn get_prefab(&self, item_prefab_id: &ItemDefId) -> &ItemDef {
        self.def_index.get(item_prefab_id).unwrap()
    }

    pub fn get_mobs_inventory_list(&self, mob_id: &MobId) -> Vec<&Item> {
        if let Some(inventory) = self.mob_inventory.get(mob_id) {
            inventory.list.iter().map(|item_id| {
                self.get(item_id)
            }).collect()
        } else {
            vec![]
        }
    }

    pub fn get_item_inventory_list(&self, item_id: &ItemId) -> Vec<&Item> {
        if let Some(inventory) = self.item_inventory.get(item_id) {
            inventory.list.iter().map(|item_id| {
                self.get(item_id)
            }).collect()
        } else {
            vec![]
        }
    }

    pub fn search(&self, room_id: &RoomId, name: &String) -> Vec<&Item> {
        match self.room_inventory.get(room_id) {
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

    pub fn move_to_mob(&mut self, mob_id: &MobId, item_id: &ItemId) {
        // remove from previous inventory
        let mut item = self.index.get(item_id).unwrap();
        // TODO: remove clone
        let location = self.item_location.get(item_id).unwrap().clone();
        let mut inventory = self.get_location_inventory_mut(&location);
        inventory.iter_mut().for_each(|inventory| inventory.remove(item_id));

        // add to new inventory
        let inventory = self.get_mob_inventory(&mob_id);
        inventory.add(*item_id);

        // mudate item location
        self.item_location.insert(*item_id, ItemLocation::Mob { mob_id: *mob_id });
    }

    pub fn get_location(&self, item_id: &ItemId) -> &ItemLocation {
        self.item_location.get(item_id).unwrap()
    }
}

impl ItemRepository {
    fn get_room_inventory(&mut self, room_id: &RoomId) -> &mut Inventory {
        self.room_inventory.entry(*room_id).or_insert(Inventory::new())
    }

    fn get_mob_inventory(&mut self, mob_id: &MobId) -> &mut Inventory {
        self.mob_inventory.entry(*mob_id).or_insert(Inventory::new())
    }

    fn get_item_inventory(&mut self, item_id: &ItemId) -> &mut Inventory {
        self.item_inventory.entry(*item_id).or_insert(Inventory::new())
    }

    fn get_location_inventory_mut(&mut self, location: &ItemLocation) -> Option<&mut Inventory> {
        match location {
            ItemLocation::Room { room_id } => {
                self.room_inventory.get_mut(room_id)
            },
            ItemLocation::Mob { mob_id } => {
                self.mob_inventory.get_mut(mob_id)
            },
            ItemLocation::Item { item_id } => {
                self.item_inventory.get_mut(item_id)
            },
            ItemLocation::Limbo => {
                None
            }
        }
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
