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

pub enum ItemPlace {
    Room {
        id: RoomId
    },
    Mob {
        id: MobId
    },
}

#[derive(Debug, Clone)]
pub struct Item {
    id: ItemId,
    typ: ItemType,
    label: String
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemDefId(pub u32);

#[derive(Debug, Clone)]
pub struct ItemDef {
    id: ItemDefId,
    typ: ItemType,
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
}

pub struct ItemRepository {
    next_item_id: NextId,
    next_item_def_id: NextId,
    index: HashMap<ItemId, Item>,
    def_index: HashMap<ItemDefId, ItemDef>,
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
            index: HashMap::new(),
            def_index: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }
}
