use crate::errors::Result;
use crate::game::loader::dto::{InventoryData, LoaderData, ObjData};
use logs::*;
use rand::seq::index::IndexVec;

pub fn migrate_to_v1(data: &mut LoaderData) -> Result<()> {
    fn migrate_to_v1_obj(data: &mut ObjData) {
        if let Some(price) = data.price.as_mut() {
            if price.price.is_none() && price.buy.is_some() {
                info!("migration v1: {:?} setting price {:?}", data.id, price.buy);
                price.price = price.buy;
            }
        }
    }

    data.objects.values_mut().for_each(migrate_to_v1_obj);
    data.prefabs.values_mut().for_each(migrate_to_v1_obj);

    data.version = 1;

    Ok(())
}

pub fn migrate_to_v2(data: &mut LoaderData) -> Result<()> {
    fn migrate(data: &mut ObjData) {
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
    }

    data.objects.values_mut().for_each(migrate);
    data.prefabs.values_mut().for_each(migrate);

    data.version = 2;

    Ok(())
}
