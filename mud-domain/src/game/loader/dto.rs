use crate::errors::{Error, Result};
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::comm::vendor_buy_item_not_found;
use crate::game::config::Config;
use crate::game::container::Container;
use crate::game::domain::{Dir, Modifier};
use crate::game::hire::Hire;
use crate::game::item::{Armor, Item, Weapon};
use crate::game::labels::Label;
use crate::game::mob::{Damage, Mob, MobId};
use crate::game::obj::Objects;
use crate::game::pos::Pos;
use crate::game::prices::{Money, Price};
use crate::game::random_rooms::{RandomRoomsCfg, RandomRoomsRepository, RandomRoomsSpawnCfg};
use crate::game::room::Room;
use crate::game::ships::Ship;
use crate::game::spawn::{Spawn, SpawnBuilder};
use crate::game::surfaces::Surface;
use crate::game::vendors::Vendor;
use crate::game::zone::Zone;
use commons::csv::FieldKind;
use commons::{DeltaTime, Either, ObjId, PlayerId, OBJ_ID_STATIC_RANGE, V2};
use logs::*;
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RoomExitData {
    pub dir: String,
    pub to: StaticId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RoomData {
    pub can_exit: Option<bool>,
    pub exits: Option<Vec<RoomExitData>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AstroBodyData {
    pub kind: String,
    pub orbit_distance: f32,
    pub jump_target_id: Option<StaticId>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SectorData {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MobData {
    pub attack: u32,
    pub defense: u32,
    pub damage_min: u32,
    pub damage_max: u32,
    pub pv: i32,
    pub pv_max: u32,
    pub xp: u32,
    pub hire_cost: Option<u32>,
    pub aggressive: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CraftData {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemFlagsData {
    pub money: Option<bool>,
    pub inventory: Option<bool>,
    pub stuck: Option<bool>,
    pub body: Option<bool>,
}

impl ItemFlagsData {
    pub fn new() -> Self {
        ItemFlagsData {
            money: None,
            inventory: None,
            stuck: None,
            body: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemWeaponData {
    pub min: u32,
    pub max: u32,
    pub calm_down: f32,
    pub attack: i32,
    pub defense: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemArmorData {
    pub defense: i32,
    pub rd: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ItemData {
    pub flags: Option<ItemFlagsData>,
    pub amount: Option<u32>,
    pub weapon: Option<ItemWeaponData>,
    pub armor: Option<ItemArmorData>,
}

impl ItemData {
    pub fn new() -> Self {
        ItemData {
            flags: None,
            amount: None,
            weapon: None,
            armor: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PosData {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Hash, Eq, Copy, Ord, PartialOrd)]
pub struct StaticId(pub u32);

impl StaticId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn is_prefab(&self) -> bool {
        self.as_u32() < OBJ_ID_STATIC_RANGE
    }
}

impl From<ObjId> for StaticId {
    fn from(id: ObjId) -> Self {
        StaticId(id.as_u32())
    }
}

impl From<&ObjId> for StaticId {
    fn from(id: &ObjId) -> Self {
        StaticId(id.as_u32())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PriceData {
    pub price: Option<u32>,
    #[deprecated]
    pub buy: Option<u32>,
    #[deprecated]
    pub sell: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MarketTradeData {
    pub tags: Vec<String>,
    pub buy_price_mult: Option<f32>,
    pub sell_price_mult: Option<f32>,
}

// #[derive(Deserialize, Serialize, Debug, Clone)]
// pub struct VendorMarketBulkMinMaxPriceData {
//     pub min: f32,
//     pub max: f32,
// }
//
// #[derive(Deserialize, Serialize, Debug, Clone)]
// pub struct VendorMarketBulkData {
//     pub item_tag: Vec<String>,
//     pub buy: Option<VendorMarketBulkMinMaxPriceData>,
//     pub sell: Option<VendorMarketBulkMinMaxPriceData>,
//     pub max_stock: f32,
//     pub stock_change_per_cycle: f32,
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MarketData {
    pub trades: Vec<MarketTradeData>,
    // bulk: Vec<VendorMarketBulkData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VendorStockData {
    pub tag: String,
    pub amount: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VendorData {
    pub market_id: Option<StaticId>,
    pub stock: Option<Vec<VendorStockData>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RandomRoomsSpawnData {
    pub level_min: Option<u32>,
    pub level_max: Option<u32>,
    pub amount: u32,
    pub spawn: SpawnData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RandomRoomsData {
    pub entrance_room_id: StaticId,
    pub entrance_dir: String,
    pub width: u32,
    pub height: u32,
    pub levels: u32,
    pub spawns: Vec<RandomRoomsSpawnData>,
    pub generated: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZoneData {
    pub random_rooms: Option<RandomRoomsData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MemoryData {
    pub knows: Vec<StaticId>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TagsData {
    pub values: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ObjData {
    pub id: Option<StaticId>,
    pub label: Option<String>,
    pub code: Option<Vec<String>>,
    pub desc: Option<String>,
    pub owned_by: Option<StaticId>,
    pub room: Option<RoomData>,
    pub astro_body: Option<AstroBodyData>,
    pub sector: Option<SectorData>,
    pub mob: Option<MobData>,
    pub pos: Option<PosData>,
    pub spawn: Option<SpawnData>,
    /// Is instantiate in same context of parent, ID is mapped
    pub parent: Option<StaticId>,
    /// Are instantiate in own context, unique ID and place as children
    pub children: Option<Vec<StaticId>>,
    pub craft: Option<CraftData>,
    pub item: Option<ItemData>,
    pub price: Option<PriceData>,
    pub vendor: Option<VendorData>,
    pub zone: Option<ZoneData>,
    pub player: Option<PlayerData>,
    pub memory: Option<MemoryData>,
    pub tags: Option<TagsData>,
    pub market: Option<MarketData>,
}

impl ObjData {
    pub fn new() -> Self {
        ObjData {
            id: None,
            label: None,
            code: None,
            desc: None,
            owned_by: None,
            room: None,
            astro_body: None,
            sector: None,
            mob: None,
            pos: None,
            spawn: None,
            parent: None,
            children: None,
            craft: None,
            item: None,
            price: None,
            vendor: None,
            zone: None,
            player: None,
            memory: None,
            tags: None,
            market: None,
        }
    }

    pub fn get_id(&self) -> StaticId {
        self.id.expect("id field not defined")
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CfgData {
    pub initial_room: StaticId,
    pub avatar_mob: StaticId,
    pub money_id: Option<StaticId>,
    pub tick: Option<u32>,
    pub total_time: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpawnData {
    pub prefab_id: StaticId,
    pub max: u32,
    pub time_min: f32,
    pub time_max: f32,
    pub locations_id: Option<Vec<StaticId>>,
    pub next_spawn: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlatData {
    pub static_id: u32,
    pub label: Option<String>,
    pub desc: Option<String>,
    pub item_weapon_attack: Option<i32>,
    pub item_weapon_defense: Option<i32>,
    pub item_weapon_damage_min: Option<u32>,
    pub item_weapon_damage_max: Option<u32>,
    pub item_weapon_calmdown: Option<f32>,
    pub price_buy: Option<u32>,
    pub item_armor_defense: Option<i32>,
    pub item_armor_rd: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerData {
    pub id: StaticId,
    pub login: String,
    pub avatar_id: StaticId,
}

// TODO: rename to snapshot data?
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoaderData {
    pub version: u32,
    pub cfg: Option<CfgData>,
    pub objects: BTreeMap<StaticId, ObjData>,
    pub prefabs: BTreeMap<StaticId, ObjData>,
}

impl LoaderData {
    pub fn new() -> Self {
        LoaderData {
            version: super::CURRENT_VERSION,
            cfg: None,
            objects: Default::default(),
            prefabs: Default::default(),
        }
    }

    /// This basic implementation only extends objects that don't exists in self, any conflict
    /// will cause a error
    pub fn extends(&mut self, data: LoaderData) -> Result<()> {
        if self.version != data.version {
            return Err("Data version mismatch".into());
        }

        if data.cfg.is_some() {
            if self.cfg.is_some() {
                return Err("Data already contains cfg".into());
            }

            self.cfg = data.cfg;
        }

        for (static_id, prefab) in data.prefabs {
            if self.prefabs.contains_key(&static_id) {
                return Err(format!("Data already contain prefab {:?}", static_id).into());
            }

            self.prefabs.insert(static_id, prefab);
        }

        for (static_id, obj) in data.objects {
            if self.objects.contains_key(&static_id) {
                return Err(format!("Data already contain object {:?}", static_id).into());
            }

            self.objects.insert(static_id, obj);
        }

        Ok(())
    }
}
