use super::container::Container;
use super::item::*;
use super::mob::*;
use crate::game::comm;
use crate::game::Outputs;
use commons::PlayerId;

pub fn pickup(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let ctx = container.get_player_context(player_id);

    let target_inventory = args.get(1);
    let target_item = args.get(2);

    match (target_inventory, target_item) {
        (Some(target_label), None) => {
            let target_inventory = container.items.get_inventory_list(ctx.room.id);
            let target_item = container.items.find_inventory(ctx.room.id, target_label);

            match target_item {
                Some(item)=> {
                    outputs.private(player_id, comm::pick_player_from_room(item.label.as_str()));
                    outputs.room(player_id, ctx.room.id, comm::pick_from_room(ctx.mob.label.as_str(), item.label.as_str()));

                    let item_id = item.id;
                    let mob_id = ctx.mob.id;
                    container.items.move_item(item_id, mob_id);
                },

                None => {
                    outputs.private(player_id, comm::pick_what(&target_inventory));
                }
            }
        },
        (Some(target_inventory_label), Some(target_item_label)) => {
            // pick up from container
            let target_inventory_item = container.items.search_inventory(ctx.room.id, target_inventory_label);
            let target_inventory_item = target_inventory_item.get(0);

            if target_inventory_item.is_none() {
                outputs.private(player_id, comm::pick_where_not_found(target_inventory_label));
                return;
            }

            let target_inventory_item = target_inventory_item.unwrap();
            let item_id = target_inventory_item.id;
            let inventory = container.items.get_inventory_list(item_id);

            if target_item.is_none() {
                outputs.private(player_id, comm::pick_what(&inventory));
                return;
            }

            // TODO: move to a util search with more powerful capabilities
            let item = inventory.iter()
                .find(|item| item.label.eq_ignore_ascii_case(target_item_label));

            match item {
                Some(item) => {
                    outputs.private(player_id, comm::pick_player_from(target_inventory_label, target_item_label));
                    outputs.room(player_id, ctx.room.id, comm::pick_from(ctx.mob.label.as_str(), target_inventory_label, target_item_label));

                    let mob_id = ctx.mob.id;
                    let item_id = item.id;
                    container.items.move_item(item_id, mob_id);
                },

                None => {
                    outputs.private(player_id, comm::pick_what(&inventory));
                }
            }
        }

        (_, _) => {
            outputs.private(player_id, comm::pick_where());
        },
    }
}

/// As a humanoid entity in mud, try to equip a item
pub fn do_equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(), ()> {
    let item = container.items.get(item_id);

    // check if mob own the item
    let has_item = container.items.get_inventory(mob_id)
        .map(|inv| inv.list.contains(&item_id))
        .unwrap_or(false);

    if !has_item {
        outputs.private_opt(player_id, comm::equip_item_invalid(item.label.as_str()));
        return Err(());
    }

    // check if can be equipped
    let can_be_equipped = item.weapon.is_some() || item.armor.is_some();

    if !can_be_equipped {
        outputs.private_opt(player_id, comm::equip_item_invalid(item.label.as_str()));
        return Err(());
    }

    // TODO: remove old equip in sample place?
    container.equips.add(mob_id, item_id);

    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob_id)?;
    outputs.private_opt(player_id, comm::equip_player_from_room(item.label.as_str()));
    outputs.room_opt(player_id, room_id,comm::equip_from_room(mob.label.as_str(), item.label.as_str()));
    Ok(())
}

pub fn do_drop(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(),()> {
    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob_id)?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);
    container.items.move_item(item_id, room_id);

    let item = container.items.get(item_id);

    outputs.private_opt(player_id, comm::drop_item(item.label.as_str()));
    outputs.room_opt(player_id, room_id, comm::drop_item_others(mob.label.as_str(), item.label.as_str()));
    Ok(())
}


#[test]
pub fn test1() {
    assert!(false);
}
