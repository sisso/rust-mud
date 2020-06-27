use crate::errors;
use crate::errors::Error;
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::comm::vendor_buy_item_not_found;
use crate::game::config::Config;
use crate::game::container::Container;
use crate::game::domain::{Dir, Modifier};
use crate::game::hire::Hire;
use crate::game::item::{Armor, Item, Weapon};
use crate::game::labels::Label;
use crate::game::mob::{Damage, Mob};
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
use commons::{DeltaTime, Either, ObjId, V2};
use logs::*;
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
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
    pub pv: u32,
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub struct StaticId(pub u32);

impl StaticId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PriceData {
    pub buy: u32,
    pub sell: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VendorData {}

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
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZoneData {
    pub random_rooms: Option<RandomRoomsData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ObjData {
    pub id: StaticId,
    pub label: String,
    pub code: Option<Vec<String>>,
    pub desc: Option<String>,
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
}

impl ObjData {
    pub fn new(id: StaticId) -> Self {
        ObjData {
            id,
            label: "".to_string(),
            code: None,
            desc: None,
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
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CfgData {
    pub initial_room: StaticId,
    pub avatar_mob: StaticId,
    pub initial_craft: Option<StaticId>,
    pub money_id: Option<StaticId>,
}

// TODO: remove HashMap, the key is probably never used
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoaderData {
    pub cfg: Option<CfgData>,
    pub objects: HashMap<StaticId, ObjData>,
    pub prefabs: HashMap<StaticId, ObjData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpawnData {
    pub prefab_id: StaticId,
    pub max: u32,
    pub time_min: f32,
    pub time_max: f32,
    pub locations_id: Option<Vec<StaticId>>,
}

impl LoaderData {
    pub fn new() -> Self {
        LoaderData {
            cfg: None,
            objects: Default::default(),
            prefabs: Default::default(),
        }
    }
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
