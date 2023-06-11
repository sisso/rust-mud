use crate::errors::{Error, Result};
use crate::game::loader::dto::{PriceData, TagsData};
use crate::game::loader::{
    dto::{InventoryData, LoaderData, ObjData},
    Migration,
};
use rand::random;
use rand::seq::index::IndexVec;

#[derive(Default)]
pub struct MigrationV2AddItemWeightAndMobInventory;

impl Migration for MigrationV2AddItemWeightAndMobInventory {
    fn version(&self) -> u32 {
        2
    }

    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        if let Some(item) = &mut data.item {
            let is_money = item.flags.as_ref().and_then(|f| f.money).unwrap_or(false);
            if !is_money && item.weight.is_none() {
                log::info!(
                    "migration v2: {:?} setting item weight to {:?}",
                    data.id,
                    1.0
                );
                item.weight = Some(1.0);
            }
        }

        if data.mob.is_some() && data.inventory.is_none() {
            let max_weight = data.mob.as_ref().unwrap().pv as f32 * 2.0;
            log::info!(
                "migration v2: {:?} setting mob inventory to {:?}",
                data.id,
                max_weight
            );
            data.inventory = Some(InventoryData {
                max_weight: Some(max_weight),
            });
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct MigrationV3FixItemsWithoutPrice;

impl Migration for MigrationV3FixItemsWithoutPrice {
    fn version(&self) -> u32 {
        3
    }

    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        let is_goods = data
            .tags
            .as_ref()
            .map(|t| t.values.iter().any(|s| s.eq("goods")))
            .unwrap_or(false);

        let has_price = data.price.is_some();

        if is_goods && !has_price {
            log::info!("migration v3: {:?} has price updated", data.id);
            data.price = Some(PriceData::new(500));
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct MigrationV4AddTags;

impl Migration for MigrationV4AddTags {
    fn version(&self) -> u32 {
        4
    }

    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        match &data.item {
            Some(idata) => {
                let mut tags = vec![];

                if idata.flags.as_ref().map(|f| f.money.unwrap_or(false)) == Some(true) {
                    tags.push("money".to_string());
                } else if idata.flags.as_ref().map(|f| f.body.unwrap_or(false)) == Some(true) {
                    tags.push("body".to_string());
                } else {
                    tags.push("item".to_string());
                }

                if idata.armor.is_some() {
                    tags.push("armor".to_string());
                }

                if idata.weapon.is_some() {
                    tags.push("weapon".to_string());
                }

                if data.tags.is_none() {
                    log::info!("migrating {:?} tags to {:?}", data.label, tags);
                    data.tags = Some(TagsData { values: tags });
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct MigrationV5CleanRandomRooms;

impl Migration for MigrationV5CleanRandomRooms {
    fn version(&self) -> u32 {
        5
    }

    fn migrate(&mut self, data: &mut LoaderData) -> Result<()> {
        let mut random_zone_id = None;

        for (id, data) in &data.objects {
            // search for zones with random rooms
            if data
                .zone
                .as_ref()
                .map(|zone| zone.random_rooms.is_some())
                .unwrap_or(false)
            {
                random_zone_id = Some(*id);
                break;
            }
        }

        if random_zone_id.is_none() {
            log::info!("no random zone found, migration complete");
            return Ok(());
        };

        log::info!("found a random zone id {:?}", random_zone_id.unwrap());

        // move all rooms into the zone
        for (id, data) in &mut data.objects {
            if !data.room.is_some() {
                continue;
            }

            // search for zones with random rooms
            if !data
                .label
                .as_ref()
                .map(|label| label.starts_with("Random room"))
                .unwrap_or(false)
            {
                continue;
            }

            data.parent = random_zone_id;
        }

        Ok(())
    }
}
