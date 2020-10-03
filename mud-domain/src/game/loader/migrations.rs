use crate::errors::Result;
use crate::game::loader::dto::{LoaderData, ObjData};
use logs::*;

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
