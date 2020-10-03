use crate::errors::*;
use crate::game::comm::VendorTradeItemDisplay;
use crate::game::container::Container;
use crate::game::item::ItemId;
use crate::game::labels::Labels;
use crate::game::loader::{dto::StaticId, Loader};
use crate::game::market::MarketTrade;
use crate::game::mob::MobId;
use crate::game::prices::Money;
use crate::game::tags::Tags;
use crate::game::{comm, inventory};
use crate::utils::strinput::StrInput;
use commons::{Either, ObjId};
use logs::*;

#[derive(Debug)]
pub struct VendorTradeObj {
    pub static_id: StaticId,
    pub base_price: Money,
    pub buy_price: Option<Money>,
    pub sell_price: Option<Money>,
}

pub fn get_vendor_trades(container: &Container, vendor_id: MobId) -> Option<&Vec<MarketTrade>> {
    let vendor = container.vendors.get(vendor_id)?;
    let market_id = vendor.market_id?;
    let market = container.markets.get(market_id)?;
    Some(&market.trades)
}

pub fn find_vendor_list(container: &Container, vendor_id: MobId) -> Result<Vec<VendorTradeObj>> {
    let mut result = vec![];

    let trades =
        get_vendor_trades(container, vendor_id).as_exception_str("vendor must have a market")?;

    for trade in trades {
        let tags_str = container
            .tags
            .resolve_str(&trade.tags)
            .as_exception_str("could not find tags")?;

        for data in container.loader.find_prefabs_by_tags_or(&tags_str) {
            let data_id = match data.id {
                None => {
                    warn!("{:?} prefab has no id", data.id);
                    continue;
                }

                Some(id) => id,
            };

            let price = match data.price.as_ref() {
                None => {
                    warn!("could not find price for {:?}", data_id);
                    continue;
                }

                Some(price) => price,
            };

            let price = price.price.as_result_exception()?;

            let trade_obj = VendorTradeObj {
                static_id: data_id,
                base_price: Money(price),
                buy_price: trade.buy_price_mult.map(|mult| Money(price).mult(mult)),
                sell_price: trade.sell_price_mult.map(|mult| Money(price).mult(mult)),
            };
            result.push(trade_obj);
        }
    }

    Ok(result)
}

pub fn vendor_items_to_vendor_list_items<'a>(
    container: &'a Container,
    vendor_list: &Vec<VendorTradeObj>,
) -> Vec<VendorTradeItemDisplay<'a>> {
    vendor_list
        .iter()
        .map(|vendor_trade| {
            let label = container
                .loader
                .get_prefab(vendor_trade.static_id)
                .unwrap()
                .label
                .as_ref()
                .unwrap()
                .as_str();

            let display = VendorTradeItemDisplay {
                label: label,
                to_buy: vendor_trade.sell_price,
                to_sell: vendor_trade.buy_price,
            };

            display
        })
        .collect()
}

pub fn list(container: &mut Container, mob_id: MobId, vendor_id: MobId) -> Result<()> {
    let list = find_vendor_list(container, vendor_id)?;
    let list = vendor_items_to_vendor_list_items(container, &list);

    let msg = comm::vendor_list(list);
    container.outputs.private(mob_id, msg);
    Ok(())
}

pub fn buy(
    container: &mut Container,
    mob_id: MobId,
    vendor_id: MobId,
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

    let vendor_items = match find_vendor_list(container, vendor_id) {
        Ok(items) => items,
        Err(e) => {
            container.outputs.private(mob_id, comm::vendor_buy_fail());
            return Err(e);
        }
    };

    let buy_price: Money = match vendor_items
        .iter()
        .filter(|i| i.static_id == item_static_id)
        .flat_map(|i| i.sell_price)
        .next()
    {
        Some(price) => price,
        None => {
            let label = data
                .label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("undefined");
            container
                .outputs
                .private(mob_id, comm::vendor_buy_item_not_found(label));

            return Err(Error::InvalidStateFailure);
        }
    };

    let mob_money = inventory::get_money(container, mob_id)?;

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

pub fn sell(
    container: &mut Container,
    mob_id: MobId,
    item_id: ObjId,
    vendor_id: MobId,
) -> Result<()> {
    let vendor_trades = get_vendor_trades(container, vendor_id);

    let sell_price = match (vendor_trades, container.prices.get(item_id)) {
        (Some(trades), Some(price)) => {
            let sell_price = trades
                .iter()
                .filter(|trade| container.tags.has_any(item_id, &trade.tags))
                .flat_map(|trade| trade.buy_price_mult)
                .map(|buy_price_mult| price.price.mult(buy_price_mult))
                .next();

            sell_price
        }

        _ => None,
    };

    // item label must be collected before contianer.remove
    let item_label = container.labels.get_label_f(item_id).to_string();

    let sell_price = match sell_price {
        Some(price) => price,
        None => {
            container.outputs.private(
                mob_id,
                comm::vendor_sell_item_not_found(item_label.as_str()),
            );
            return Err(Error::InvalidStateFailure);
        }
    };

    // add money
    inventory::add_money(container, mob_id, sell_price).map_err(|err| {
        container
            .outputs
            .private(mob_id, comm::vendor_operation_fail());
        err
    })?;

    // remove item
    container.remove(item_id);

    // print messages
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
