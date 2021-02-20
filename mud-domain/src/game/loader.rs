mod hocon_parser;

use crate::errors::{AsResult, Error, Result};
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::config::Config;
use crate::game::container::Container;
use crate::game::domain::{Dir, Modifier};
use crate::game::hire::Hire;
use crate::game::item::{Armor, Item, Weapon, Weight};
use crate::game::labels::Label;
use crate::game::loader::hocon_parser::{HParser, ParseError};
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
use commons::{DeltaTime, Either, ObjId, Tick, TotalTime, V2};
use logs::*;
use rand::random;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::{Borrow, BorrowMut};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub mod dto;
mod migrations;

use crate::game::ai::{Ai, AiCommand, AiRepo};
use crate::game::inventory::Inventory;
use crate::game::loader::migrations::*;
use crate::game::market::{Market, MarketTrade};
use commons::jsons::JsonValueExtra;
use dto::*;
use std::io::Write;

const MIGRATION_LATEST_VERSION: u32 = 3;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub duplicate_ids: Vec<StaticId>,
    pub mismatch_ids: Vec<(StaticId, StaticId)>,
}

impl ValidationResult {
    pub fn assert_valid(&self) -> Result<()> {
        if !self.mismatch_ids.is_empty() {
            return Err(Error::Error(format!(
                "conflict object id {:?} and {:?}",
                self.mismatch_ids[0].0, self.mismatch_ids[0].1
            )));
        }

        if !self.duplicate_ids.is_empty() {
            return Err(Error::Error(format!(
                "duplicate prefab id {:?}",
                self.duplicate_ids[0]
            )));
        }

        Ok(())
    }
}

trait Migration {
    fn version(&self) -> u32;
    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        Ok(())
    }
    fn migrate(&mut self, data: &mut LoaderData) -> Result<()> {
        for (_, data) in &mut data.prefabs {
            self.migrate_obj_or_prefab(data)?;
        }
        for (_, data) in &mut data.objects {
            self.migrate_obj_or_prefab(data)?;
        }

        Ok(())
    }
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
        // TODO: autogenerate id?
        assert!(
            data.id.is_some(),
            "data can only be inserted with a valid id"
        );
        assert!(!self.index.contains_key(&data.get_id()));
        debug!("{:?} adding prefab {:?}", data.id, data);
        self.index.insert(data.get_id(), data);
    }

    pub fn get_prefab(&self, id: StaticId) -> Option<&ObjData> {
        self.index.get(&id)
    }

    pub fn get_prefab_label(&self, id: StaticId) -> Option<&str> {
        let value = self.get_prefab(id)?.label.as_ref()?.as_str();
        Some(value)
    }

    pub fn get_prefab_weight(&self, id: StaticId) -> Option<Weight> {
        self.get_prefab(id)?.item.as_ref()?.weight
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

    pub fn find_prefabs_by_tags_or<'a>(
        &'a self,
        tags: &'a Vec<&'a str>,
    ) -> impl Iterator<Item = &ObjData> + 'a {
        self.index.values().filter(move |data| {
            data.tags
                .as_ref()
                .filter(|data_tags| {
                    let found = data_tags
                        .values
                        .iter()
                        .find(|t| tags.contains(&t.as_str()))
                        .is_some();

                    found
                })
                .is_some()
        })
    }
}

// TODO: organize fields, it is a mess
/// static fields
impl Loader {
    pub fn spawn_at(
        container: &mut Container,
        static_id: StaticId,
        parent_id: ObjId,
    ) -> Result<ObjId> {
        let obj_id = Loader::instantiate(container, static_id)?;
        container.locations.set(obj_id, parent_id);
        Ok(obj_id)
    }

    pub fn instantiate(container: &mut Container, static_id: StaticId) -> Result<ObjId> {
        debug!("instantiate prefab {:?}", static_id);

        let mut references = HashMap::new();

        // create objects
        let obj_id = container.objects.create();
        trace!("{:?} creating prefab of {:?}", obj_id, static_id);
        references.insert(static_id, obj_id);

        // instantiate children
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
            let data = container
                .loader
                .get_prefab(static_id)
                .expect("static id not found")
                .clone();

            Loader::apply_data(container, obj_id, &data, &references)?;
        }

        Ok(obj_id)
    }

    /// Resolve the static id to a ObjId by first searching in reference_map and then in container
    fn get_by_static_id(
        objects: &Objects,
        ref_map: &HashMap<StaticId, ObjId>,
        static_id: StaticId,
    ) -> Result<ObjId> {
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

    /// Failed results should be usually thread as a Exception, as it is not atomic and could leak
    /// components
    pub fn apply_data(
        container: &mut Container,
        obj_id: ObjId,
        data: &ObjData,
        references: &HashMap<StaticId, ObjId>,
    ) -> Result<()> {
        macro_rules! get_ref {
            ($res:expr) => {
                Loader::get_by_static_id(&container.objects, &references, $res).unwrap()
            };
        }

        debug!("{:?} apply prefab {:?}", obj_id, data.id);

        if let Some(parent) = &data.parent {
            let parent_id = Loader::get_by_static_id(&container.objects, &references, *parent)?;
            container.locations.set(obj_id, parent_id)
        }

        if let Some(label) = data.label.as_ref() {
            // TODO: simplify
            let code = data
                .code
                .clone()
                .map(|i| i.first().cloned())
                .and_then(|identity| identity)
                .unwrap_or(label.clone());

            let desc = data.desc.clone().unwrap_or("".to_string());

            container.labels.add(Label {
                id: obj_id,
                label: label.clone(),
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

            if let Some(old_value) = container.astro_bodies.upsert(body) {
                warn!("{:?} already have a astro_body: {:?}", obj_id, old_value);
            }
        }

        if let Some(_craft) = &data.craft {
            container.ships.add(Ship::new(obj_id));
        }

        if let Some(_surfaces) = &data.sector {
            container.surfaces.add(Surface {
                id: obj_id,
                size: 10,
                is_3d: false,
            });
        }

        if let Some(mob_data) = &data.mob {
            let mut mob = Mob::new(obj_id);
            mob.attributes.attack = mob_data.attack;
            mob.attributes.defense = mob_data.defense;
            mob.attributes.pv.current = mob_data.pv;
            mob.attributes.pv.max = mob_data.pv_max;
            mob.attributes.damage.max = mob_data.damage_max;
            mob.attributes.damage.min = mob_data.damage_min;
            mob.xp = mob_data.xp;
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
            if let Some(next) = spawn_data.next_spawn {
                spawn.next = TotalTime(next);
            }
            container.spawns.add(spawn)?;
        }

        if let Some(data_item) = &data.item {
            let mut item = Item::new(obj_id);

            item.amount = data_item.amount.unwrap_or(1);
            item.weight = data_item.weight;

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
            let data_price = data.price.expect("price should have price");
            let price = Price::new(obj_id, Money(data_price));
            container.prices.add(price);
        }

        if let Some(vendor_data) = &data.vendor {
            let mut vendor = Vendor::new(obj_id);
            if let Some(market_id) = vendor_data.market_id {
                let id = Loader::get_by_static_id(&container.objects, &references, market_id)
                    .expect("could not find market id");
                vendor.market_id = Some(id);
            }
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
                            "locations_id is not supported for spawn in random maps, it should be empty"
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
                    // TODO: deep?
                    .add(RandomRoomsCfg {
                        id: obj_id,
                        entrance_id: entrance_id,
                        entrance_dir: Dir::parse(rr_data.entrance_dir.as_str()).unwrap(),
                        // TODO: add seed
                        seed: 0,
                        levels: rr_data.levels,
                        width: rr_data.width,
                        height: rr_data.height,
                        spawns: spawns,
                        generated: rr_data.generated,
                    })
                    .unwrap();
            }
        }

        if let Some(pos) = &data.pos {
            container.pos.set(obj_id, V2::new(pos.x, pos.y));
        }

        if let Some(owned_id) = &data.owned_by {
            let owner_id = get_ref!(*owned_id);

            container.ownership.set_owner(obj_id, owner_id);
        }

        if let Some(player_data) = &data.player {
            let player_id = get_ref!(player_data.id);
            let avatar_id = get_ref!(player_data.avatar_id);

            container
                .players
                .create(player_id, player_data.login.clone(), avatar_id);
        }

        if let Some(memory) = &data.memory {
            let mut memory_ids: Vec<ObjId> = memory
                .knows
                .iter()
                .map(|static_id| get_ref!(*static_id))
                .collect();

            memory_ids.sort_unstable();

            container.memories.add_all(obj_id, memory_ids)?;
        }

        if let Some(tags) = &data.tags {
            for tag_str in &tags.values {
                let id = container.tags.get_id(tag_str.as_str());
                container.tags.add(obj_id, id);
            }
        }

        if let Some(market_data) = &data.market {
            let tags = &mut container.tags;

            let market = Market {
                id: obj_id,
                trades: market_data
                    .trades
                    .iter()
                    .map(|trade_data| MarketTrade {
                        tags: trade_data
                            .tags
                            .iter()
                            .map(|tag| tags.get_id(tag.as_str()))
                            .collect(),
                        buy_price_mult: trade_data.buy_price_mult,
                        sell_price_mult: trade_data.sell_price_mult,
                    })
                    .collect(),
            };
            container
                .markets
                .add(market)
                .expect("fail to insert market");
        }

        if let Some(inventory_data) = &data.inventory {
            let mut inv = Inventory::new(obj_id);
            inv.max_weight = inventory_data.max_weight;
            container.inventories.add(inv).unwrap();
        }

        if let Some(ai_data) = &data.ai {
            let ai = Loader::parse_ai(obj_id, ai_data);
            container.ai.add_or_update(ai).unwrap();
        }

        if let Some(children) = data.children.clone() {
            for static_id in children.into_iter() {
                trace!("{:?} spawn children {:?}", obj_id, static_id);
                Loader::spawn_at(container, static_id, obj_id)?;
            }
        }

        Ok(())
    }

    pub fn load_hocon(container: &mut Container, buffer: &str) -> Result<()> {
        let data: Result<LoaderData> = HParser::load_hocon_str(buffer).map_err(|e| {
            let msg = format!("{:?}", e);
            Error::Error(msg)
        });

        Loader::load_data(container, data?)
    }

    pub fn read_csv_files<T: AsRef<Path>>(data: &mut LoaderData, files: &Vec<T>) -> Result<()> {
        let mut flat_data = vec![];

        for file in files {
            info!("reading file {:?}", file.as_ref());
            let buffer = std::fs::read_to_string(file).unwrap();
            let list = Loader::read_csv(buffer.as_str())?;
            flat_data.extend(list);
        }

        Loader::parse_flat_data(data, flat_data)
    }

    pub fn read_csv(buffer: &str) -> Result<Vec<FlatData>> {
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
                .map_err(|err| Error::Exception(format!("{}", err)))?;
            result.push(data);
        }

        Ok(result)
    }

    pub fn load_from_csv(container: &mut Container, buffer: &str) -> Result<()> {
        let flat_values = Loader::read_csv(buffer)?;
        let mut data = LoaderData::new();
        Loader::parse_flat_data(&mut data, flat_values)?;
        Loader::load_data(container, data)
    }

    pub fn parse_flat_data(root_data: &mut LoaderData, list: Vec<FlatData>) -> Result<()> {
        for data in list {
            let static_id = StaticId(data.static_id);
            let mut obj = ObjData::new();
            obj.id = Some(static_id);

            if let Some(label) = data.label {
                obj.label = Some(label);
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
                obj.price = Some(PriceData {
                    price: Some(price_buy),
                    buy: None,
                    sell: None,
                });
            }

            trace!("reading into {:?}", obj);
            root_data.prefabs.insert(obj.get_id(), obj);
        }

        Ok(())
    }

    /// Algorithm
    ///
    /// 1. Load all files and resolve variables
    /// 2. Validate content
    /// 3. Add all prefabs
    /// 4. Instantiate all static data
    pub fn load_folders(container: &mut Container, folder: &Path) -> Result<()> {
        let data = Loader::read_folders(folder)?;
        Loader::load_data(container, data)
    }

    pub fn read_json(data: &mut LoaderData, json_file: &Path) -> Result<()> {
        info!("reading file {:?}", json_file);
        let file = std::fs::File::open(json_file)?;
        let mut new_data = serde_json::from_reader(std::io::BufReader::new(file))?;
        Loader::migrate(&mut new_data);
        data.extends(new_data)
    }

    /// Read a snapshot file
    pub fn read_snapshot(snapshot_file: &Path) -> Result<LoaderData> {
        let file = std::fs::File::open(snapshot_file)?;
        let data = serde_json::from_reader(std::io::BufReader::new(file))?;
        Ok(data)
    }

    /// Write data as snapshot file
    pub fn write_snapshot(snapshot_file: &Path, data: &LoaderData) -> Result<()> {
        let mut jvalue = serde_json::to_value(data)?;
        jvalue.strip_nulls();

        let value = serde_json::to_string_pretty(&jvalue)?;
        let mut file = std::fs::File::create(snapshot_file)?;
        file.write_all(value.as_bytes())?;
        Ok(())
    }

    pub fn read_files(files: Vec<&Path>) -> Result<LoaderData> {
        let mut data = LoaderData::new();

        let json_files = files
            .iter()
            .filter(|path| path.to_string_lossy().ends_with(".json"))
            .collect::<Vec<_>>();

        for json_file in json_files {
            Loader::read_json(&mut data, json_file)?;
        }

        // load csv files
        let csv_files = files
            .iter()
            .filter(|path| path.to_string_lossy().ends_with(".csv"))
            .collect::<Vec<_>>();

        Loader::read_csv_files(&mut data, &csv_files)?;

        // load hocon files
        let conf_files = files
            .iter()
            .filter(|path| path.to_string_lossy().ends_with(".conf"))
            .collect();

        HParser::load_hocon_files(&mut data, &conf_files).map_err(|e| match e {
            ParseError::HoconError { error, hint } => {
                warn!("Fail loading data {:?} {}", error, hint);
                Error::Error(format!("Loading data {:?}: {}", error, hint))
            }

            e => Error::Error(format!("Fail loading data {:?}", e)),
        })?;

        Ok(data)
    }

    pub fn read_folders(root_path: &Path) -> Result<LoaderData> {
        if !root_path.exists() {
            return Err(Error::Error(
                "configuration folder do not exists".to_string(),
            ));
        }

        let files = Loader::list_files(root_path)?;
        let files: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        Loader::read_files(files)
    }

    // TODO: to fs-utils?
    fn list_files(root_path: &Path) -> Result<Vec<PathBuf>> {
        let mut result = vec![];
        let list: std::fs::ReadDir = std::fs::read_dir(root_path)?;
        for entry in list {
            let path = entry?.path();
            if path.is_dir() {
                result.extend(Loader::list_files(&path)?);
            } else {
                result.push(path);
            }
        }

        Ok(result)
    }

    pub fn create_snapshot(container: &Container) -> Result<LoaderData> {
        let mut data = LoaderData::new();

        data.cfg = Some(CfgData {
            // TODO should be option
            initial_room: container.config.initial_room.unwrap().into(),
            // TODO should be option
            avatar_mob: container.config.avatar_id.unwrap(),
            money_id: container.config.money_id,
            tick: Some(container.time.tick.as_u32()),
            total_time: Some(container.time.total.as_seconds_f64()),
        });

        for prefab in container.loader.list_prefabs() {
            data.prefabs.insert(prefab.get_id(), prefab.clone());
        }

        for obj_id in container.objects.list() {
            let obj_data = Loader::snapshot_obj(&container, *obj_id)?;
            data.objects.insert(obj_id.into(), obj_data);
        }

        Ok(data)
    }

    pub fn snapshot_obj(container: &Container, id: ObjId) -> Result<ObjData> {
        macro_rules! true_or_none {
            ($res:expr) => {
                if $res {
                    Some(true)
                } else {
                    None
                }
            };
        }

        macro_rules! nonempty_or_none {
            ($res:expr) => {
                if $res.is_empty() {
                    None
                } else {
                    Some($res)
                }
            };
        }

        let mut obj_data = ObjData::new();
        obj_data.id = Some(id.into());
        obj_data.prefab_id = container.objects.get_prefab_id(id);

        if let Some(label) = container.labels.get(id) {
            // Hack some fields to make it compatible with original values
            obj_data.label = Some(label.label.clone());
            if !label.desc.is_empty() {
                obj_data.desc = Some(label.desc.clone());
            }

            if label.code != label.label {
                obj_data.code = Some(vec![label.code.clone()]);
            }
        }

        if let Some(room) = container.rooms.get(id) {
            let exits: Vec<_> = room
                .exits
                .iter()
                .map(|(dir, room_id)| RoomExitData {
                    dir: dir.as_str().to_string(),
                    to: room_id.into(),
                })
                .collect();

            obj_data.room = Some(RoomData {
                // TODO: hack for migration
                can_exit: if room.can_exit { Some(true) } else { None },
                exits: Some(exits), // if exits.is_empty() { None } else { Some(exits) },
            });
        }

        if let Some(_sector) = container.surfaces.get(id) {
            obj_data.sector = Some(SectorData {});
        }

        if let Some(parent) = container.locations.get(id) {
            obj_data.parent = Some(parent.into());
        }

        if let Some(_zone) = container.zones.get(id) {
            let random_room_data = if let Some(random_room) = container.random_rooms.get(id) {
                let spanws = random_room
                    .spawns
                    .iter()
                    .map(|i| RandomRoomsSpawnData {
                        level_min: i.level_min,
                        level_max: i.level_max,
                        amount: i.amount,
                        spawn: SpawnData {
                            prefab_id: i.spawn_builder.prefab_id,
                            max: i.spawn_builder.max,
                            time_min: i.spawn_builder.delay_min.as_seconds_f32(),
                            time_max: i.spawn_builder.delay_max.as_seconds_f32(),
                            // random maps have empty locations
                            locations_id: None,
                            next_spawn: None,
                            ai_overwrite: i.spawn_builder.ai_overwrite.clone(),
                        },
                    })
                    .collect();

                Some(RandomRoomsData {
                    entrance_room_id: random_room.entrance_id.into(),
                    entrance_dir: random_room.entrance_dir.as_str().to_string(),
                    width: random_room.width,
                    height: random_room.height,
                    levels: random_room.levels,
                    spawns: spanws,
                    // entrance is always created during load
                    generated: random_room.generated,
                })
            } else {
                None
            };

            obj_data.zone = Some(ZoneData {
                random_rooms: random_room_data,
            });
        }

        if let Some(pos) = container.pos.get_pos(id) {
            obj_data.pos = Some(PosData { x: pos.x, y: pos.y });
        }

        if let Some(astro_body) = container.astro_bodies.get(id) {
            obj_data.astro_body = Some(AstroBodyData {
                kind: astro_body.kind.as_str().to_string(),
                orbit_distance: astro_body.orbit_distance,
                jump_target_id: astro_body.jump_target_id.map(|value| value.into()),
            })
        }

        if let Some(spawn) = container.spawns.get(id) {
            let locations: Vec<_> = spawn.locations_id.iter().map(|id| id.into()).collect();

            let spawn_data = SpawnData {
                prefab_id: spawn.prefab_id,
                max: spawn.max,
                time_min: spawn.delay.min.as_seconds_f32(),
                time_max: spawn.delay.max.as_seconds_f32(),
                locations_id: nonempty_or_none!(locations),
                next_spawn: Some(spawn.next.as_seconds_f64()),
                ai_overwrite: spawn.ai_overwrite.clone(),
            };

            obj_data.spawn = Some(spawn_data);
        }

        if let Some(mob) = container.mobs.get(id) {
            let hire_cost = if let Some(hire) = container.hires.get(id) {
                Some(hire.cost.as_u32())
            } else {
                None
            };

            obj_data.mob = Some(MobData {
                attack: mob.attributes.attack,
                defense: mob.attributes.defense,
                damage_min: mob.attributes.damage.min,
                damage_max: mob.attributes.damage.max,
                pv: mob.attributes.pv.current,
                pv_max: mob.attributes.pv.max,
                xp: mob.xp,
                hire_cost: hire_cost,
            })
        }

        if let Some(item) = container.items.get(id) {
            let flags = ItemFlagsData {
                money: true_or_none!(item.flags.is_money),
                inventory: true_or_none!(item.flags.is_inventory),
                stuck: true_or_none!(item.flags.is_stuck),
                body: true_or_none!(item.flags.is_corpse),
            };

            let weapon = if let Some(weapon) = &item.weapon {
                Some(ItemWeaponData {
                    min: weapon.damage.min,
                    max: weapon.damage.max,
                    calm_down: weapon.calm_down.as_seconds_f32(),
                    attack: weapon.attack.as_i32(),
                    defense: 0,
                })
            } else {
                None
            };

            let armor = if let Some(armor) = &item.armor {
                Some(ItemArmorData {
                    defense: armor.defense.as_i32(),
                    rd: armor.rd,
                })
            } else {
                None
            };

            let amount = if item.amount == 1 {
                None
            } else {
                Some(item.amount)
            };

            obj_data.item = Some(ItemData {
                flags: Some(flags),
                amount: amount,
                weapon: weapon,
                armor: armor,
                weight: item.weight,
            });
        }

        if let Some(owner) = container.ownership.get_owner(id) {
            obj_data.owned_by = Some(owner.into());
        }

        if let Some(player) = container.players.get(id) {
            obj_data.player = Some(PlayerData {
                id: StaticId(player.id.0),
                login: player.login.clone(),
                avatar_id: player.mob_id.into(),
            });
        }

        if let Some(ship) = container.ships.get(id) {
            obj_data.craft = Some(CraftData {});
        }

        if let Some(memory) = container.memories.get(id) {
            let mut memory_ids: Vec<StaticId> =
                memory.know_ids.iter().map(|id| id.into()).collect();

            memory_ids.sort_by_key(|i| i.as_u32());

            obj_data.memory = Some(MemoryData { knows: memory_ids });
        }

        if let Some(vendor) = container.vendors.get(id) {
            obj_data.vendor = Some(VendorData {
                market_id: vendor.market_id.map(|id| id.into()),
                stock: None,
            });
        }

        if let Some(tags) = container.tags.get_tags(id) {
            let mut values = vec![];
            for tag_id in tags.iter() {
                match container.tags.get_str(*tag_id) {
                    Some(tag_str) => values.push(tag_str.to_string()),
                    None => {
                        warn!("could not found tag_id {:?}", tag_id);
                    }
                }
            }
            values.sort_unstable();
            obj_data.tags = Some(TagsData { values });
        }

        if let Some(market) = container.markets.get(id) {
            obj_data.market = Some(MarketData {
                trades: market
                    .trades
                    .iter()
                    .map(|trade| MarketTradeData {
                        tags: container
                            .tags
                            .resolve_strings(&trade.tags)
                            .expect("fail to resolve tags"),
                        buy_price_mult: trade.buy_price_mult,
                        sell_price_mult: trade.sell_price_mult,
                    })
                    .collect(),
            });
        }

        if let Some(inventory) = container.inventories.get(id) {
            obj_data.inventory = Some(InventoryData {
                max_weight: inventory.max_weight,
            });
        }

        if let Some(price) = container.prices.get(id) {
            obj_data.price = Some(PriceData {
                price: Some(price.price.as_u32()),
                buy: None,
                sell: None,
            });
        }

        if let Some(ai) = container.ai.get(id) {
            obj_data.ai = Some(Loader::serialize_ai(ai));
        }

        Ok(obj_data)
    }

    pub fn load_data(container: &mut Container, mut data: LoaderData) -> Result<()> {
        Loader::validate_and_normalize(&mut data)?.assert_valid()?;

        Loader::migrate(&mut data)?;

        // add prefabs
        for (_k, v) in data.prefabs {
            container.loader.add_prefab(v);
        }

        // add objects
        Loader::load_all(container, data.objects)?;

        // update configurations with references
        match data.cfg {
            Some(CfgData {
                initial_room,
                avatar_mob,
                money_id,
                tick,
                total_time,
            }) => {
                container.config.initial_room = Some(ObjId(initial_room.as_u32()));
                container.config.avatar_id = Some(avatar_mob);
                container.config.money_id = money_id;

                match (tick, total_time) {
                    (Some(tick), Some(total_time)) => {
                        container.time.set(Tick(tick), TotalTime(total_time));
                    }
                    (None, None) => {}
                    _other => panic!("Unexpect time configuration {:?}", data.cfg),
                }
            }
            _ => {}
        }

        // initialize objects
        crate::game::system::random_room_generators_system::init(container);
        crate::game::inventory_service::update_all_current_inventory(container);

        Ok(())
    }

    pub fn validate_and_normalize(data: &mut LoaderData) -> Result<ValidationResult> {
        let mut ids = HashSet::new();
        let mut result = ValidationResult {
            duplicate_ids: vec![],
            mismatch_ids: vec![],
        };

        // normalize data.id if missing
        // check for mismatch id
        // check for duplicated id
        for (static_id, data) in &mut data.objects {
            if let Some(id) = data.id {
                if id != *static_id {
                    result.mismatch_ids.push((*static_id, id));
                }
            } else {
                data.id = Some(*static_id);
            }

            if !ids.insert(data.id) {
                result.duplicate_ids.push(*static_id);
            }
        }

        for (static_id, data) in &mut data.prefabs {
            if let Some(id) = data.id {
                if id != *static_id {
                    result.mismatch_ids.push((*static_id, id));
                }
            } else {
                data.id = Some(*static_id);
            }

            if !ids.insert(data.id) {
                result.duplicate_ids.push(*static_id);
            }
        }

        Ok(result)
    }

    pub fn migrate(data: &mut LoaderData) -> Result<()> {
        info!("checking for data migration for {:?}", data.version);

        let migrations: Vec<Box<dyn Migration>> = vec![
            Box::new(MigrationV1::default()),
            Box::new(MigrationV2::default()),
            Box::new(MigrationV3::default()),
        ];

        for mut migration in migrations {
            if data.version < migration.version() {
                info!(
                    "migrating data v{} to v{} started",
                    data.version,
                    migration.version()
                );
                migration.migrate(data)?;
                data.version = migration.version();
                info!("migrating data to v{} complete", data.version);
            }
        }

        if data.version != MIGRATION_LATEST_VERSION {
            return Err(Error::Exception(format!(
                "Invalid data version after migration"
            )));
        }

        Ok(())
    }

    fn load_all(container: &mut Container, objects: BTreeMap<StaticId, ObjData>) -> Result<()> {
        for (key, _) in &objects {
            container.objects.insert(ObjId(key.as_u32()))?;
        }

        for (id, data) in &objects {
            let mut empty_references = Default::default();
            Loader::apply_data(container, ObjId(id.as_u32()), data, &mut empty_references)?;
        }

        Ok(())
    }

    fn spawn_data_to_spawn_builder(data: &SpawnData) -> SpawnBuilder {
        SpawnBuilder {
            max: data.max,
            delay_min: DeltaTime(data.time_min),
            delay_max: DeltaTime(data.time_max),
            prefab_id: data.prefab_id,
            next: data.next_spawn.as_ref().map(|time| TotalTime(*time)),
            ai_overwrite: data.ai_overwrite.clone(),
        }
    }

    pub fn apply_ai_data(ai_repo: &mut AiRepo, mob_id: MobId, ai_data: &AiData) -> Result<()> {
        let ai = Loader::parse_ai(mob_id, ai_data);
        ai_repo.add_or_update(ai);
        Ok(())
    }

    fn parse_ai(obj_id: ObjId, ai_data: &AiData) -> Ai {
        let command = if ai_data.command_aggressive.unwrap_or(false) {
            AiCommand::Aggressive
        } else if let Some(target_id) = ai_data.command_follow_and_protect {
            AiCommand::FollowAndProtect { target_id }
        } else if let Some(haul) = &ai_data.command_haul {
            AiCommand::Hauler {
                from: haul.from_id.clone(),
                to: haul.to_id.clone(),
                wares: haul.targets.clone(),
            }
        } else if let Some(patrol_data) = &ai_data.command_aggressive_patrol_home {
            AiCommand::AggressivePatrolHome {
                distance: patrol_data.distance,
            }
        } else {
            AiCommand::Idle
        };

        Ai {
            id: obj_id,
            command: command,
            commandable: ai_data.commandable.unwrap_or(false),
        }
    }

    fn serialize_ai(ai: &Ai) -> AiData {
        AiData {
            command_aggressive: if ai.command == AiCommand::Aggressive {
                Some(true)
            } else {
                None
            },
            command_follow_and_protect: match ai.command {
                AiCommand::FollowAndProtect { target_id } => Some(target_id),
                _ => None,
            },
            command_haul: match &ai.command {
                AiCommand::Hauler { from, to, wares } => Some(AiCommandHaulData {
                    from_id: *from,
                    to_id: *to,
                    targets: wares.clone(),
                }),
                _ => None,
            },
            command_aggressive_patrol_home: match &ai.command {
                AiCommand::AggressivePatrolHome { distance } => {
                    Some(AiCommandAggressivePatrolHomeData {
                        distance: *distance,
                    })
                }
                _ => None,
            },
            commandable: if ai.commandable { Some(true) } else { None },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::comm::item_corpse_appears_in_room;

    fn load_and_snapshot(obj: ObjData) -> ObjData {
        let mut container = Container::new();
        let mut load_data = LoaderData::new();
        let obj_id = obj.id.unwrap();
        load_data.objects.insert(obj_id, obj);
        Loader::load_data(&mut container, load_data).expect("fail to load data");
        Loader::snapshot_obj(&container, ObjId(obj_id.as_u32())).expect("fail to create snapshot")
    }

    fn assert_data_eq(mut value: ObjData, mut expected: ObjData) {
        // Manually check for fields that will not match in a simple jvalue check
        expected.children = None;

        if let Some(room) = &mut expected.room {
            if room.exits.is_none() {
                room.exits = Some(vec![]);
            }
        }

        // Hardcoded situations

        // Random room will create a new entrance. Maybe should not be dynamic? Still space would have it
        if let Some(expected_label) = expected.label.as_ref() {
            if expected_label == "Dungeon Entrance" {
                // should have 1 entrance in config
                expected.room.as_mut().unwrap().exits = None;
                // should have 2 entrance, the one from config + the new random rooms
                value.room.as_mut().unwrap().exits = None;
            }

            if expected_label == "Dungeon area" {
                expected
                    .zone
                    .as_mut()
                    .unwrap()
                    .random_rooms
                    .as_mut()
                    .unwrap()
                    .generated = true;
            }
        }

        // check all other fields to be equals
        crate::utils::test::assert_json_eq(&value, &expected);
    }

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
            let error_msg = format!("fail to parse {:?}", folder);
            Loader::load_folders(&mut container, folder.as_path()).expect(error_msg.as_str());
        }
    }

    #[test]
    fn test_snapshot_all_objects() -> std::result::Result<(), Box<dyn std::error::Error>> {
        for folder in list_data_folders_for_test() {
            println!("loading {}", folder.to_string_lossy());
            let data = Loader::read_folders(folder.as_path())?;

            let mut container = Container::new();
            Loader::load_data(&mut container, data.clone())?;

            for (id, obj_data) in data.objects {
                let new_obj_data = Loader::snapshot_obj(&container, ObjId(id.as_u32()))?;

                assert_data_eq(new_obj_data, obj_data);
            }
        }

        Ok(())
    }

    #[test]
    fn test_serialize_craft() {
        let mut data = ObjData::new();
        data.id = Some(StaticId(0));
        data.craft = Some(CraftData {});

        let result = load_and_snapshot(data.clone());
        assert_data_eq(data, result);
    }

    #[test]
    fn test_serialize_vendor() {
        let mut data = ObjData::new();
        data.id = Some(StaticId(0));
        data.vendor = Some(VendorData {
            market_id: None,
            stock: None,
        });

        let result = load_and_snapshot(data.clone());
        assert_data_eq(data, result);
    }

    #[test]
    fn test_find_prefab_by_tags_or() {
        let mut loader = Loader::new();

        let mut data1 = ObjData::new();
        data1.id = Some(StaticId(0));
        data1.tags = Some(TagsData {
            values: vec!["tag_a".to_string()],
        });
        loader.add_prefab(data1);

        let mut data2 = ObjData::new();
        data2.id = Some(StaticId(1));
        data2.tags = Some(TagsData {
            values: vec!["tag_b".to_string()],
        });
        loader.add_prefab(data2);

        let search_tags = vec!["tag_a"];
        let mut result = loader.find_prefabs_by_tags_or(&search_tags);
        assert!(result.next().is_some());
        assert!(result.next().is_none());
    }
}
