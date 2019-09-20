use super::container::Container;
use super::item::*;
use super::room::*;
use super::mob::*;
use crate::game::comm;
use crate::game::player::PlayerId;
use crate::game::Outputs;

pub fn pickup(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let ctx = container.get_player_context(player_id);

    let target_inventory = args.get(1);
    let target_item = args.get(2);

    if target_inventory.is_none() {
        outputs.private(player_id.clone(), comm::pick_where());
        return;
    }

    let target_inventory = target_inventory.unwrap();
    let target_inventory_item = container.items.search(&ctx.avatar.room_id, target_inventory);
    let target_inventory_item = target_inventory_item.get(0);

    if target_inventory_item.is_none() {
        outputs.private(player_id.clone(), comm::pick_where_not_found(target_inventory));
        return;
    }

    let target_inventory_item = target_inventory_item.unwrap();
    let item_id = target_inventory_item.id;
    let inventory = container.items.get_item_inventory_list(&item_id);

    if target_item.is_none() {
        outputs.private(player_id.clone(), comm::pick_what(&inventory));
        return;
    }

    let target_item= target_item.unwrap();

    let item = inventory.iter()
        .find(|item| item.label.eq_ignore_ascii_case(target_item));

    if item.is_none() {
        outputs.private(player_id, comm::pick_what(&inventory));
        return;
    }

    outputs.private(player_id, comm::pick_player_from(target_inventory, target_item));
    outputs.room(player_id, ctx.room.id ,comm::pick_from(ctx.avatar.label.as_str(), target_inventory, target_item));

    let mob_id = ctx.avatar.id;
    let item_id = item.unwrap().id;
    container.items.move_to_mob(&mob_id, &item_id);
}
