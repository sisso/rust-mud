use serde::Deserialize;
use std::collections::HashMap;

pub mod scenery_space;
pub mod scenery_fantasy;
pub mod hocon_loader;

#[derive(Deserialize, Debug)]
pub struct RoomExitData {
    pub dir: String,
    pub to: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct RoomData {
    pub airlock: Option<bool>,
    pub exits: Option<Vec<RoomExitData>>
}

#[derive(Deserialize, Debug)]
pub struct PlanetData {

}

#[derive(Deserialize, Debug)]
pub struct SectorData {

}

#[derive(Deserialize, Debug)]
pub struct MobData {
    pub attack: u32,
    pub defense: u32,
    pub damage_min: u32,
    pub damage_max: u32,
    pub pv: u32
}

#[derive(Deserialize, Debug)]
pub struct PosData {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct StaticId(pub String);

impl StaticId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Deserialize, Debug)]
pub struct ObjData {
    pub label: String,
    pub code: Option<Vec<String>>,
    pub desc: Option<String>,
    pub room: Option<RoomData>,
    pub planet: Option<PlanetData>,
    pub sector: Option<SectorData>,
    pub mob: Option<MobData>,
    pub pos: Option<PosData>,
    pub parent: Option<StaticId>,
}

#[derive(Deserialize, Debug)]
pub struct CfgData {
    pub initial_room: StaticId,
    pub avatar_mob: StaticId,
    pub initial_craft: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub cfg: Option<CfgData>,
    pub objects: HashMap<StaticId, ObjData>,
    pub prefabs: HashMap<StaticId, ObjData>,
}

