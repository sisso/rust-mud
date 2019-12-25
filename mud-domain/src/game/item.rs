use super::comm;
use crate::game::container::Ctx;
use commons::*;
use logs::*;
use std::collections::HashMap;
use crate::errors::{Error, Result, AsResult};

pub type ItemId = ObjId;
pub type ItemPrefabId = ObjId;

#[derive(Debug, Clone)]
pub struct ItemFlags {
    /// can hold more items
    pub is_inventory: bool,
    /// can not be pickup
    pub is_stuck: bool,
    /// someone body
    pub is_corpse: bool,
    pub is_gold: bool,
}

impl ItemFlags {
    pub fn new() -> Self {
        ItemFlags {
            is_inventory: false,
            is_stuck: false,
            is_corpse: false,
            is_gold: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub decay: Option<TotalTime>,
    pub amount: u32,
    pub item_def_id: Option<ItemPrefabId>,
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
    pub flags: ItemFlags,
}

impl Item {
    pub fn new(id: ItemId) -> Self {
        Item {
            id,
            decay: None,
            amount: 1,
            item_def_id: None,
            weapon: None,
            armor: None,
            flags: ItemFlags::new(),
        }
    }

    pub fn can_equip(&self) -> bool {
        self.weapon.is_some() || self.armor.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub damage_min: u32,
    pub damage_max: u32,
    pub reload: DeltaTime,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub rd: u32,
    pub dp: i32,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub location: ObjId,
    pub list: Vec<ItemId>,
}

pub struct ItemRepository {
    index: HashMap<ItemId, Item>,
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            index: HashMap::new(),
        }
    }

    pub fn exists(&self, item_id: ItemId) -> bool {
        self.index.contains_key(&item_id)
    }

    pub fn get(&self, item_id: ItemId) -> Option<&Item> {
        self.index.get(&item_id)
    }

    pub fn add(&mut self, item: Item) {
        if self.index.contains_key(&item.id) {
            panic!()
        }

        if item.amount <= 0 {
            panic!()
        }

        debug!("{:?} add item {:?}", item.id, item);

        // update index
        self.index.insert(item.id, item);
    }

    pub fn remove(&mut self, item_id: ItemId) -> Option<Item> {
        self.index.remove(&item_id).map(|item| {
            debug!("{:?} item removed ", item_id);
            item
        })
    }

    pub fn list(&self) -> Vec<ItemId> {
        self.index.keys().map(|i| *i).collect()
    }

    //    pub fn save(&self, save: &mut dyn Save) {
    //        use serde_json::json;
    //
    //        for (id, obj) in self.index.iter() {
    //            let obj_json = json!({
    //                "kind": obj.kind.0,
    //                "label": obj.label,
    //                "decay": obj.decay.map(|i| i.0),
    //                "amount": obj.amount,
    //                "definition_id": obj.item_def_id.map(|i| i.0)
    //            });
    //
    //            save.add(id.0, "item", obj_json);
    //        }
    //
    //        for (id, (location, inventory)) in self.inventory.iter().enumerate() {
    //            let location_json = match location {
    //                ObjId::Limbo => json!({"kind": "limbo"}),
    //                ObjId::Mob { mob_id } => json!({"kind": "mob", "mob_id": mob_id.0 }),
    //                ObjId::Room { room_id } => json!({"kind": "room", "room_id": room_id.0 }),
    //                ObjId::Item { item_id } => json!({"kind": "item", "item_id": item_id.0 }),
    //            };
    //
    //            let obj_json = json!({
    //                "location": location_json,
    //                "items": inventory.list.iter().map(|i| i.0).collect::<Vec<u32>>()
    //            });
    //
    //            save.add(id as u32, "inventory", obj_json);
    //        }
    //    }
}

pub fn run_tick(ctx: &mut Ctx) {
    ctx.container.items.list().into_iter().for_each(|i| {
        let _ = run_for(ctx, i);
    });
}

fn run_for(ctx: &mut Ctx, item_id: ItemId) -> Result<()> {
    let item = ctx.container.items.get(item_id).as_result()?;

    if let Some(decay) = item.decay {
        // TODO: Only decay items on ground?
        let location_id = ctx.container.locations.get(item.id).as_result()?;
        if ctx.container.rooms.exists(location_id) && decay.is_before(ctx.container.time.total) {
            info!("{:?} removed by decay", item.id);

            let label = ctx.container.labels.get_label_f(item.id);

            let msg = comm::item_body_disappears(label);
            ctx.outputs.broadcast(None, location_id, msg);
            ctx.container.remove(item_id);
            ctx.container.items.remove(item_id);
        }
    }

    Ok(())
}
