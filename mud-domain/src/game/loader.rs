mod hocon_parser;

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
use crate::game::loader::hocon_parser::{HParser, ParseError};
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
    level_min: Option<u32>,
    level_max: Option<u32>,
    amount: u32,
    spawn: SpawnData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RandomRoomsData {
    entrance_room_id: StaticId,
    entrance_dir: String,
    width: u32,
    height: u32,
    levels: u32,
    spawns: Vec<RandomRoomsSpawnData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ZoneData {
    random_rooms: Option<RandomRoomsData>,
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
    static_id: u32,
    label: Option<String>,
    desc: Option<String>,
    item_weapon_attack: Option<i32>,
    item_weapon_defense: Option<i32>,
    item_weapon_damage_min: Option<u32>,
    item_weapon_damage_max: Option<u32>,
    item_weapon_calmdown: Option<f32>,
    price_buy: Option<u32>,
    item_armor_defense: Option<i32>,
    item_armor_rd: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Loader {
    index: HashMap<StaticId, ObjData>,
}

impl Loader {
    pub fn new() -> Self {
        Loader {
            index: Default::default(),
        }
    }

    pub fn add_prefab(&mut self, data: ObjData) {
        assert!(!self.index.contains_key(&data.id));
        debug!("{:?} adding prefab {:?}", data.id, data);
        self.index.insert(data.id, data);
    }

    pub fn get_prefab(&self, id: StaticId) -> Option<&ObjData> {
        self.index.get(&id)
    }

    pub fn find_prefabs_by_parent(&self, id: StaticId) -> Vec<StaticId> {
        self.index
            .iter()
            .filter(|(_, data)| {
                data.parent
                    .map(|parent_id| parent_id == id)
                    .unwrap_or(false)
            })
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn find_deep_prefabs_by_parents(&self, static_id: StaticId) -> Vec<StaticId> {
        let mut result = vec![];
        let mut queue = vec![];

        queue.push(static_id);

        while !queue.is_empty() {
            let current = queue.pop().unwrap();
            for child_id in self.find_prefabs_by_parent(current) {
                if result.contains(&child_id) {
                    panic!(
                        "recursive reference found on {:?} when searching for {:?}",
                        child_id, static_id
                    );
                }

                result.push(child_id);
                queue.push(child_id);
            }
        }

        result
    }

    pub fn list_prefabs<'a>(&'a self) -> impl Iterator<Item = &ObjData> + 'a {
        self.index.values()
    }
}

// TODO: organize fields, is a mess
/// static fields
impl Loader {
    pub fn spawn_at(
        container: &mut Container,
        static_id: StaticId,
        parent_id: ObjId,
    ) -> errors::Result<ObjId> {
        let obj_id = Loader::instantiate(container, static_id)?;
        container.locations.set(obj_id, parent_id);
        Ok(obj_id)
    }

    pub fn instantiate(container: &mut Container, static_id: StaticId) -> errors::Result<ObjId> {
        debug!("instantiate prefab {:?}", static_id);

        let mut references = HashMap::new();

        // create objects
        let obj_id = container.objects.create();
        trace!("{:?} creating prefab of {:?}", obj_id, static_id);
        references.insert(static_id, obj_id);

        let children_prefabs = container.loader.find_deep_prefabs_by_parents(static_id);
        for child_static_id in children_prefabs {
            let child_id = container.objects.create();
            trace!(
                "instantiate prefab {:?} child {:?} with id {:?}",
                static_id,
                child_static_id,
                child_id
            );
            references.insert(child_static_id, child_id);
        }

        // initialize all
        for (&static_id, &obj_id) in &references {
            Loader::apply_prefab(container, obj_id, Either::Right(static_id), &references)?;
        }

        Ok(obj_id)
    }

    /// Resolve the static id to a ObjId by first searching in reference_map and then in container
    fn get_by_static_id(
        objects: &Objects,
        ref_map: &HashMap<StaticId, ObjId>,
        static_id: StaticId,
    ) -> errors::Result<ObjId> {
        // search from map and fallback to real ObjId
        ref_map
            .get(&static_id)
            .cloned()
            .or_else(|| {
                let id = ObjId(static_id.as_u32());
                if objects.exists(id) {
                    Some(id)
                } else {
                    None
                }
            })
            .ok_or_else(|| Error::Failure(format!("Static id {:?} can not be resolved", static_id)))
    }

    // TODO: make it atomic: success and change or no change
    fn apply_prefab(
        container: &mut Container,
        obj_id: ObjId,
        data: Either<&ObjData, StaticId>,
        references: &HashMap<StaticId, ObjId>,
    ) -> errors::Result<()> {
        let data: &ObjData = match data {
            Either::Left(data) => data,
            Either::Right(static_id) => container
                .loader
                .get_prefab(static_id)
                .ok_or(Error::NotFoundStaticId(static_id))?,
        };

        debug!("{:?} apply prefab {:?}", obj_id, data.id);

        container.objects.set_static_id(obj_id, data.id)?;

        if let Some(parent) = &data.parent {
            let parent_id = Loader::get_by_static_id(&container.objects, &references, *parent)?;
            container.locations.set(obj_id, parent_id)
        }

        {
            let label = data.label.clone();
            // TODO: simplify
            let code = data
                .code
                .clone()
                .map(|i| i.first().cloned())
                .and_then(|o| o)
                .unwrap_or(label.clone());
            let desc = data.desc.clone().unwrap_or("".to_string());

            container.labels.add(Label {
                id: obj_id,
                label,
                code,
                desc,
            });
        }

        if let Some(astro_body) = &data.astro_body {
            let orbit_distance = astro_body.orbit_distance;
            let kind = match astro_body.kind.as_ref() {
                "star" => AstroBodyKind::Star,
                "asteroid_field" => AstroBodyKind::AsteroidField,
                "ship" => AstroBodyKind::Ship,
                "station" => AstroBodyKind::Station,
                "planet" => AstroBodyKind::Planet,
                "moon" => AstroBodyKind::Moon,
                "jump_gate" => AstroBodyKind::JumpGate,
                other => panic!("invalid astro body type {:?}", other),
            };

            let mut body = AstroBody::new(obj_id, orbit_distance, kind);

            match (kind, astro_body.jump_target_id) {
                (AstroBodyKind::JumpGate, Some(target_static_id)) => {
                    let target_id = Loader::get_by_static_id(
                        &container.objects,
                        &references,
                        target_static_id,
                    )?;
                    body.jump_target_id = Some(target_id);
                }

                (AstroBodyKind::JumpGate, None) => {
                    warn!("{:?} jump_target_id must be defined for Jump", obj_id);
                }

                (_, Some(_)) => {
                    warn!("{:?} jump_target_id is only available to jump kind", obj_id);
                }

                _ => {}
            }

            container.astro_bodies.insert(body).unwrap();
        }

        if let Some(_craft) = &data.craft {
            container.ships.add(Ship::new(obj_id));
        }

        if let Some(_surfaces) = &data.sector {
            container.sectors.add(Surface {
                id: obj_id,
                size: 10,
                is_3d: false,
            });
        }

        if let Some(mob_data) = &data.mob {
            let mut mob = Mob::new(obj_id);
            mob.attributes.attack = mob_data.attack;
            mob.attributes.defense = mob_data.defense;
            mob.attributes.pv.current = mob_data.pv as i32;
            mob.attributes.pv.max = mob_data.pv;
            mob.attributes.damage.max = mob_data.damage_max;
            mob.attributes.damage.min = mob_data.damage_min;
            mob.xp = mob_data.xp;
            mob.aggressive = mob_data.aggressive.unwrap_or(false);
            container.mobs.add(mob);

            if let Some(hire_cost) = mob_data.hire_cost {
                let mut hire = Hire::new(obj_id);
                hire.cost = Money(hire_cost);
                container.hires.add(hire).unwrap();
            }
        }

        if let Some(room_data) = &data.room {
            let mut room = Room::new(obj_id);
            room.can_exit = room_data.can_exit.unwrap_or(false);

            if let Some(exists) = &room_data.exits {
                for i in exists {
                    let dir = Dir::parse(i.dir.as_str()).unwrap();
                    let to_id = Loader::get_by_static_id(&container.objects, &references, i.to)?;

                    room.exits.push((dir, to_id));
                }
            }

            container.rooms.add(room);
        }

        if let Some(spawn_data) = &data.spawn {
            let builder = Loader::spawn_data_to_spawn_builder(spawn_data);

            let mut locations_id = vec![];
            if let Some(locations) = &spawn_data.locations_id {
                for static_id in locations {
                    let location_id =
                        Loader::get_by_static_id(&container.objects, &references, *static_id)?;
                    locations_id.push(location_id);
                }
            }

            let mut spawn = builder.create_spawn(obj_id);
            spawn.locations_id = locations_id;
            container.spawns.add(spawn)?;
        }

        if let Some(data_item) = &data.item {
            let mut item = Item::new(obj_id);

            item.amount = data_item.amount.unwrap_or(1);

            if let Some(flags) = &data_item.flags {
                item.flags.is_corpse = flags.body.unwrap_or(false);
                item.flags.is_money = flags.money.unwrap_or(false);
                item.flags.is_inventory = flags.inventory.unwrap_or(false);
                item.flags.is_stuck = flags.stuck.unwrap_or(false);
            }

            if let Some(armor_data) = &data_item.armor {
                let mut armor = Armor::new();
                armor.defense = Modifier(armor_data.defense);
                armor.rd = armor_data.rd;
                item.armor = Some(armor);
            }

            if let Some(weapon_data) = &data_item.weapon {
                let mut weapon = Weapon::new();
                weapon.attack = Modifier(weapon_data.attack);
                weapon.calm_down = DeltaTime(weapon_data.calm_down);
                weapon.damage = Damage {
                    min: weapon_data.min,
                    max: weapon_data.max,
                };
                item.weapon = Some(weapon);
            }

            container.items.add(item);
        }

        if let Some(data) = &data.price {
            let price = Price::new(obj_id, Money(data.buy), Money(data.sell));
            container.prices.add(price);
        }

        if let Some(_data) = &data.vendor {
            let vendor = Vendor::new(obj_id);
            container.vendors.add(vendor);
        }

        if let Some(zone_data) = &data.zone {
            container.zones.add(Zone { id: obj_id }).unwrap();

            if let Some(rr_data) = &zone_data.random_rooms {
                let entrance_id = Loader::get_by_static_id(
                    &container.objects,
                    &references,
                    rr_data.entrance_room_id,
                )
                .unwrap();

                let spawns = rr_data
                    .spawns
                    .iter()
                    .map(|spawn_data| {
                        assert!(
                            spawn_data.spawn.locations_id.is_none(),
                            "locations_id is not supported for spawn in random maps"
                        );

                        let spawn_builder = Loader::spawn_data_to_spawn_builder(&spawn_data.spawn);
                        RandomRoomsSpawnCfg {
                            amount: spawn_data.amount,
                            level_min: spawn_data.level_min,
                            level_max: spawn_data.level_max,
                            spawn_builder: spawn_builder,
                        }
                    })
                    .collect();

                container
                    .random_rooms
                    .add(RandomRoomsCfg {
                        id: obj_id,
                        entrance_id: entrance_id,
                        entrance_dir: Dir::parse(rr_data.entrance_dir.as_str()).unwrap(),
                        seed: 0,
                        width: rr_data.width,
                        height: rr_data.height,
                        spawns: spawns,
                    })
                    .unwrap();
            }
        }

        if let Some(children) = data.children.clone() {
            for static_id in children.into_iter() {
                trace!("{:?} spawn children {:?}", obj_id, static_id);
                Loader::spawn_at(container, static_id, obj_id)?;
            }
        }

        Ok(())
    }

    pub fn load_hocon(container: &mut Container, buffer: &str) -> errors::Result<()> {
        let data: errors::Result<LoaderData> = HParser::load_from_str(buffer).map_err(|e| {
            let msg = format!("{:?}", e);
            errors::Error::Error(msg)
        });

        Loader::load_data(container, data?)
    }

    pub fn read_csv_files(data: &mut LoaderData, files: &Vec<&Path>) -> errors::Result<()> {
        let mut flat_data = vec![];

        for file in files {
            debug!("reading file {:?}", file);
            let buffer = std::fs::read_to_string(file).unwrap();
            let list = Loader::read_csv(buffer.as_str())?;
            flat_data.extend(list);
        }

        Loader::parse_flat_data(data, flat_data)
    }

    pub fn read_csv(buffer: &str) -> errors::Result<Vec<FlatData>> {
        let csv = commons::csv::parse_csv(buffer);
        let tables = commons::csv::csv_strings_to_tables(&csv).expect("fail to parse tables");

        let mut parsers = HashMap::new();
        parsers.insert("static_id", FieldKind::U32);
        parsers.insert("item_weapon_attack", FieldKind::I32);
        parsers.insert("item_weapon_defense", FieldKind::I32);
        parsers.insert("item_weapon_damage_max", FieldKind::U32);
        parsers.insert("item_weapon_damage_min", FieldKind::U32);
        parsers.insert("item_weapon_calmdown", FieldKind::F32);
        parsers.insert("item_armor_defense", FieldKind::I32);
        parsers.insert("item_armor_rd", FieldKind::I32);
        parsers.insert("price_buy", FieldKind::U32);

        let mut result = vec![];
        let json_list = commons::csv::tables_to_jsonp(&tables, &parsers).unwrap();
        for value in json_list {
            let data: FlatData = serde_json::from_value(value)
                .map_err(|err| errors::Error::Exception(format!("{}", err)))?;
            result.push(data);
        }

        Ok(result)
    }

    pub fn load_from_csv(container: &mut Container, buffer: &str) -> errors::Result<()> {
        let flat_values = Loader::read_csv(buffer)?;
        let mut data = LoaderData::new();
        Loader::parse_flat_data(&mut data, flat_values)?;
        Loader::load_data(container, data)
    }

    pub fn parse_flat_data(root_data: &mut LoaderData, list: Vec<FlatData>) -> errors::Result<()> {
        for data in list {
            let static_id = StaticId(data.static_id);
            let mut obj = ObjData::new(static_id);

            if let Some(label) = data.label {
                obj.label = label;
                obj.desc = data.desc;
            }

            let is_weapon = data.item_weapon_damage_min.is_some();
            let is_armor = data.item_armor_rd.is_some();
            let is_item = is_weapon || is_armor;
            if is_item {
                let mut item = ItemData::new();

                if is_weapon {
                    let weapon = ItemWeaponData {
                        min: data.item_weapon_damage_min.unwrap(),
                        max: data.item_weapon_damage_max.unwrap(),
                        calm_down: data.item_weapon_calmdown.unwrap(),
                        attack: data.item_weapon_attack.unwrap(),
                        defense: data.item_weapon_defense.unwrap(),
                    };

                    item.weapon = Some(weapon);
                }

                if is_armor {
                    let armor = ItemArmorData {
                        defense: data.item_armor_defense.unwrap(),
                        rd: data.item_armor_rd.unwrap(),
                    };

                    item.armor = Some(armor);
                }

                obj.item = Some(item);
            }

            if let Some(price_buy) = data.price_buy {
                let price_sell = price_buy / 2;
                obj.price = Some(PriceData {
                    buy: price_buy,
                    sell: price_sell,
                });
            }

            trace!("reading into {:?}", obj);
            root_data.prefabs.insert(obj.id, obj);
        }

        Ok(())
    }

    /// Algorithm
    ///
    /// 1. Load all files and resolve variables
    /// 2. Validate content
    /// 3. Add all prefabs
    /// 4. Instantiate all static data
    pub fn load_folders(container: &mut Container, folder: &Path) -> errors::Result<()> {
        let data = Loader::read_folders(folder)?;
        Loader::load_data(container, data)
    }

    pub fn read_folders(root_path: &Path) -> errors::Result<LoaderData> {
        if !root_path.exists() {
            return Err(Error::Error(
                "configuration folder do not exists".to_string(),
            ));
        }

        let files = Loader::list_files(root_path)?;

        let mut data = LoaderData::new();

        // load csv files
        let csv_files = files
            .iter()
            .filter(|path| path.to_string_lossy().ends_with(".csv"))
            .map(|p| Path::new(p.to_str().unwrap()))
            .collect::<Vec<_>>();

        Loader::read_csv_files(&mut data, &csv_files)?;

        // load hocon files
        let conf_files = files
            .iter()
            .filter(|path| path.to_string_lossy().ends_with(".conf"))
            .map(|p| Path::new(p.to_str().unwrap()))
            .collect();

        HParser::load_files(&mut data, &conf_files).map_err(|e| match e {
            ParseError::HoconError { error, hint } => {
                warn!("Fail loading data {:?} {}", error, hint);
                Error::Error(format!("Loading data {:?}: {}", error, hint))
            }

            e => Error::Error(format!("Fail loading data {:?}", e)),
        })?;

        Ok(data)
    }

    // TODO: make it recursive
    fn list_files(root_path: &Path) -> errors::Result<Vec<PathBuf>> {
        let mut result = vec![];
        let list: std::fs::ReadDir = std::fs::read_dir(root_path)?;
        for entry in list {
            let path = entry?.path();
            result.push(path);
        }

        Ok(result)
    }

    pub fn load_data(container: &mut Container, data: LoaderData) -> errors::Result<()> {
        Loader::validate(&data)?;

        // add prefabs
        for (_k, v) in data.prefabs {
            container.loader.add_prefab(v);
        }

        // instantiate static data
        Loader::initialize_all(container, data.objects)?;

        // update configurations with references
        match data.cfg {
            Some(CfgData {
                initial_room,
                avatar_mob,
                initial_craft: _,
                money_id,
            }) => {
                container.config.initial_room = Some(ObjId(initial_room.as_u32()));
                container.config.avatar_id = Some(avatar_mob);
                container.config.money_id = money_id;
            }
            _ => {}
        }

        // initialize objects
        crate::game::system::random_room_generators_system::init(container);

        Ok(())
    }

    fn validate(data: &LoaderData) -> errors::Result<()> {
        let mut ids = HashSet::new();

        for (_static_id, data) in data.objects.iter() {
            if !ids.insert(data.id) {
                return Err(Error::Error(format!("duplicate object id {:?}", data.id)));
            }
        }

        for (_static_id, data) in data.prefabs.iter() {
            if !ids.insert(data.id) {
                return Err(Error::Error(format!("duplicate prefab id {:?}", data.id)));
            }
        }

        Ok(())
    }

    fn initialize_all(
        container: &mut Container,
        objects: HashMap<StaticId, ObjData>,
    ) -> errors::Result<()> {
        for (key, _) in &objects {
            container.objects.insert(ObjId(key.as_u32()))?;
        }

        for (id, data) in &objects {
            let mut empty_references = Default::default();
            Loader::apply_prefab(
                container,
                ObjId(id.as_u32()),
                Either::Left(data),
                &mut empty_references,
            )?;
        }

        Ok(())
    }

    fn spawn_data_to_spawn_builder(data: &SpawnData) -> SpawnBuilder {
        SpawnBuilder {
            max: data.max,
            delay_min: DeltaTime(data.time_min),
            delay_max: DeltaTime(data.time_max),
            prefab_id: data.prefab_id,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::comm::item_corpse_appears_in_room;

    #[test]
    pub fn initialize_with_spawn() {
        let buffer = r#"objects.sector_1_dune_palace {
    id: 0,
    label: "Palace"
    desc: "The greate Palace of Dune"
    room: {
      exits: [
        {dir: "s", to: ${objects.sector_1_dune_landing_pad.id} }
      ]
    }
}

objects.sector_1_dune_landing_pad {
    id: 1,
    label: "Landing pad"
    desc: "City landing pad."
    room: {
      landing_pad: true
      exits: [
        {dir: "n", to: ${objects.sector_1_dune_palace.id} }
      ]
    }
    children: [2]
}

prefabs.control_panel {
    id: 2,
    label: "Control Panel",
}

prefabs.control_panel_command_1 {
    id: 3,
    label: "Command 1",
    parent: 2,
    room: {
        exits: [ {dir: "s", to: 4  } ]
    }
}

prefabs.control_panel_command_2 {
    id: 4,
    label: "Command 2",
    parent: 2,
    room: {
        exits: [ {dir: "n", to: 3  } ]
    }
}"#;

        let mut container = Container::new();
        Loader::load_hocon(&mut container, buffer).unwrap();

        let landing_pad_id = ObjId(1);

        let landing_pad = container.rooms.get(landing_pad_id).unwrap();
        assert_eq!(ObjId(0), landing_pad.exits.first().unwrap().1);

        let at_landing_pad = container
            .locations
            .list_at(landing_pad_id)
            .collect::<Vec<_>>();
        assert_eq!(1, at_landing_pad.len());

        let control_panel_id = *at_landing_pad.first().unwrap();
        let panel_str = container.labels.get_label_f(control_panel_id);
        assert_eq!("Control Panel", panel_str);

        let at_control_panel = container
            .locations
            .list_at(control_panel_id)
            .collect::<Vec<_>>();
        assert_eq!(2, at_control_panel.len());

        let mut command1_id = None;
        let mut command2_id = None;

        for id in at_control_panel {
            let label = container.labels.get_label(id).unwrap();
            match label {
                "Command 1" => command1_id = Some(id),
                "Command 2" => command2_id = Some(id),
                other => panic!("Unexpected {:?}", other),
            }
        }

        assert!(command1_id.is_some());
        assert!(command2_id.is_some());

        let room = container.rooms.get(command1_id.unwrap()).unwrap();
        assert_eq!(command2_id.unwrap(), room.exits.first().unwrap().1);

        let room = container.rooms.get(command2_id.unwrap()).unwrap();
        assert_eq!(command1_id.unwrap(), room.exits.first().unwrap().1);
    }

    fn list_data_folders_for_test() -> Vec<std::path::PathBuf> {
        let mut result = vec![];
        let path = Path::new("../data");
        for file in std::fs::read_dir(path).unwrap() {
            let file = file.unwrap();
            if file.metadata().unwrap().is_dir() {
                result.push(file.path());
            }
        }
        result
    }

    #[test]
    fn test_read_all_data_folders() {
        for folder in list_data_folders_for_test() {
            let mut container = Container::new();
            Loader::load_folders(&mut container, &folder).unwrap();
        }
    }
}
