use std::collections::HashMap;
use crate::game::domain::NextId;
use crate::game::room::RoomId;
use crate::game::mob::MobId;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemType(pub u32);

pub const ITEM_TYPE_COIN: ItemType = ItemType(0);
pub const ITEM_TYPE_BODY: ItemType = ItemType(1);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub typ: ItemType,
    pub label: String
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
    list: Vec<Item>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            list: vec![]
        }
    }

    pub fn add(&mut self, item: Item) {
        self.list.push(item);
    }
}

pub struct ItemRepository {
    next_item_id: NextId,
    next_item_def_id: NextId,
//    index: HashMap<ItemId, Item>,
    room_inventory: HashMap<RoomId, Inventory>,
    mob_inventory: HashMap<MobId, Inventory>,
    def_index: HashMap<ItemDefId, ItemDef>,
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
//            index: HashMap::new(),
            room_inventory: Default::default(),
            mob_inventory: Default::default(),
            def_index: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }

    pub fn add_to_room(&mut self, item: Item, room_id: RoomId) {
        let inventory = self.get_room_inventory(room_id);
        inventory.add(item);

//        self.add(item);
    }

    pub fn list_at(&self, room_id: &RoomId) -> Vec<&Item> {
        match self.room_inventory.get(room_id) {
            Some(inventoy) => inventoy.list.iter().collect(),
            None => vec![],
        }
    }

//    fn add(&mut self, item: Item) {
//        self.index.insert(item.id, item);
//    }

    fn get_room_inventory(&mut self, room_id: RoomId) -> &mut Inventory {
        self.room_inventory.entry(room_id).or_insert(Inventory::new())
    }
}
