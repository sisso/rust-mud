use crate::errors::*;
use crate::game::comm::VendorListItem;
use crate::game::container::Container;
use crate::game::item::ItemId;
use crate::game::labels::Labels;
use crate::game::loader::{dto::StaticId, Loader};
use crate::game::mob::MobId;
use crate::game::prices::Money;
use crate::game::{comm, inventory};
use crate::utils::strinput::StrInput;
use commons::ObjId;
use logs::*;

pub fn list(container: &mut Container, mob_id: MobId, _vendor_id: MobId) -> Result<()> {
    let list = container
        .loader
        .list_prefabs()
        .flat_map(|prefab| {
            let label = prefab.label.as_str();
            let price = prefab.price.as_ref()?;

            Some(VendorListItem {
                label,
                buy: (price.buy as u32).into(),
                sell: (price.sell as u32).into(),
            })
        })
        .collect();

    let msg = comm::vendor_list(list);
    container.outputs.private(mob_id, msg);
    Ok(())
}

// TODO: boilerplate send you rewards
pub fn buy(
    container: &mut Container,
    mob_id: MobId,
    _vendor_id: MobId,
    item_static_id: StaticId,
) -> Result<ItemId> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        warn!("{:?} player has no location", mob_id);
        container.outputs.private(mob_id, comm::vendor_buy_fail());
        Error::InvalidStateFailure
    })?;

    let data = match container.loader.get_prefab(item_static_id) {
        Some(value) => value,
        None => {
            warn!("static id {:?} not found", item_static_id);
            container.outputs.private(mob_id, comm::vendor_buy_fail());
            return Err(Error::NotFoundException);
        }
    };

    let buy_price: Money = match &data.price {
        Some(item) => item.buy.into(),
        None => {
            warn!("{:?} has no price to be bought", item_static_id);
            container.outputs.private(mob_id, comm::vendor_buy_fail());
            return Err(Error::InvalidStateFailure);
        }
    };

    let mob_money = inventory::get_money(container, mob_id).map_err(|err| {
        warn!("{:?} fail to get mob money", mob_id);
        container.outputs.private(mob_id, comm::vendor_buy_fail());
        err
    })?;

    if mob_money.as_u32() < buy_price.as_u32() {
        container.outputs.private(
            mob_id,
            comm::vendor_buy_you_have_not_enough_money(mob_money, buy_price),
        );
        return Err(Error::InvalidArgumentFailure);
    }

    let new_mob_money = inventory::remove_money(container, mob_id, buy_price).map_err(|err| {
        warn!("{:?} fail remove mob money", mob_id);
        container.outputs.private(mob_id, comm::vendor_buy_fail());
        err
    })?;

    let item_id = Loader::spawn_at(container, item_static_id, mob_id).map_err(|err| {
        warn!(
            "{:?} fail to spawn bought item for {:?}",
            item_static_id, mob_id
        );
        container.outputs.private(mob_id, comm::vendor_buy_fail());
        err
    })?;

    let mob_label = container.labels.get_label_f(mob_id);
    let item_label = container.labels.get_label_f(item_id);

    container.outputs.private(
        mob_id,
        comm::vendor_buy_success(item_label, buy_price, new_mob_money),
    );
    container.outputs.broadcast(
        Some(mob_id),
        location_id,
        comm::vendor_buy_success_others(mob_label, item_label),
    );

    Ok(item_id)
}

pub fn sell(container: &mut Container, mob_id: MobId, item_id: ObjId) -> Result<()> {
    let sell_price = match container.prices.get(item_id) {
        Some(data) => data.sell,

        None => {
            let label = container.labels.get_label_f(item_id);

            container
                .outputs
                .private(mob_id, comm::vendor_sell_item_not_found(label));

            return Err(Error::NotFoundFailure);
        }
    };

    inventory::add_money(container, mob_id, sell_price).map_err(|err| {
        container
            .outputs
            .private(mob_id, comm::vendor_operation_fail());
        err
    })?;

    // label must be collect before remove of item
    let item_label = container.labels.get_label_f(item_id).to_string();

    container.remove(item_id);

    let mob_label = container.labels.get_label_f(mob_id);
    let location_id = container.locations.get(mob_id).unwrap();
    container.outputs.private(
        mob_id,
        comm::vendor_sell_item(item_label.as_str(), sell_price),
    );
    container.outputs.broadcast(
        Some(mob_id),
        location_id,
        comm::vendor_sell_item_for_others(mob_label, item_label.as_str()),
    );

    Ok(())
}
