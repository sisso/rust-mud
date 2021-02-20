use crate::errors::Result;
use crate::game::loader::dto::PriceData;
use crate::game::loader::{
    dto::{InventoryData, LoaderData, ObjData},
    Migration,
};
use logs::*;
use rand::seq::index::IndexVec;

/// Migrate price.buy/price.sell by price.price
pub struct MigrationV1 {}

impl Default for MigrationV1 {
    fn default() -> Self {
        MigrationV1 {}
    }
}

impl Migration for MigrationV1 {
    fn version(&self) -> u32 {
        1
    }

    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        if let Some(price) = data.price.as_mut() {
            if price.price.is_none() && price.buy.is_some() {
                info!("migration v1: {:?} setting price {:?}", data.id, price.buy);
                price.price = price.buy;
                price.buy = None;
                price.sell = None;
            }
        }

        Ok(())
    }
}

/// Add item weight and mob inventory
pub struct MigrationV2;

impl Default for MigrationV2 {
    fn default() -> Self {
        MigrationV2 {}
    }
}

impl Migration for MigrationV2 {
    fn version(&self) -> u32 {
        2
    }

    fn migrate_obj_or_prefab(&mut self, data: &mut ObjData) -> Result<()> {
        if let Some(item) = &mut data.item {
            let is_money = item.flags.as_ref().and_then(|f| f.money).unwrap_or(false);
            if !is_money && item.weight.is_none() {
                info!(
                    "migration v2: {:?} setting item weight to {:?}",
                    data.id, 1.0
                );
                item.weight = Some(1.0);
            }
        }

        if data.mob.is_some() && data.inventory.is_none() {
            let max_weight = data.mob.as_ref().unwrap().pv as f32 * 2.0;
            info!(
                "migration v2: {:?} setting mob inventory to {:?}",
                data.id, max_weight
            );
            data.inventory = Some(InventoryData {
                max_weight: Some(max_weight),
            });
        }
        Ok(())
    }
}

/// fix items without price
pub struct MigrationV3;

impl Default for MigrationV3 {
    fn default() -> Self {
        MigrationV3 {}
    }
}

impl Migration for MigrationV3 {
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
            info!("migration v3: {:?} has price updated", data.id);
            data.price = Some(PriceData::new(500));
        }

        Ok(())
    }
}
