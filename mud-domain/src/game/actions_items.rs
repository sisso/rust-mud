use super::container::Container;
use super::item::*;
use super::room::*;
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
            let target_inventory = container.items.get_inventory_list(ItemLocation::Room { room_id: ctx.room.id });
            // TODO: move to a util search with more powerful capabilities
            let target_item = target_inventory.iter().find(|item| {
                item.label.eq_ignore_ascii_case(target_label)
            });

            match target_item {
                Some(item)=> {
                    outputs.private(player_id, comm::pick_player_from_room(item.label.as_str()));
                    outputs.room(player_id, ctx.room.id, comm::pick_from_room(ctx.avatar.label.as_str(), item.label.as_str()));

                    let item_id = item.id;
                    let mob_id = ctx.avatar.id;
                    container.items.move_item(item_id, ItemLocation::Mob { mob_id });
                },

                None => {
                    outputs.private(player_id, comm::pick_what(&target_inventory));
                }
            }
        },
        (Some(target_inventory_label), Some(target_item_label)) => {
            // pick up from container
            let target_inventory_item = container.items.search(&ctx.avatar.room_id, target_inventory_label);
            let target_inventory_item = target_inventory_item.get(0);

            if target_inventory_item.is_none() {
                outputs.private(player_id, comm::pick_where_not_found(target_inventory_label));
                return;
            }

            let target_inventory_item = target_inventory_item.unwrap();
            let item_id = target_inventory_item.id;
            let inventory = container.items.get_inventory_list(ItemLocation::Item { item_id });

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
                    outputs.room(player_id, ctx.room.id, comm::pick_from(ctx.avatar.label.as_str(), target_inventory_label, target_item_label));

                    let mob_id = ctx.avatar.id;
                    let item_id = item.id;
                    container.items.move_item(item_id, ItemLocation::Mob { mob_id });
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

pub fn equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    match do_equip(container, player_id, args) {
        Ok(EquipAction::Success { item_id }) => {
            let ctx = container.get_player_context(player_id);
            let item = container.items.get(item_id);
            outputs.private(player_id, comm::equip_player_from_room(item.label.as_str()));
            outputs.room(player_id, ctx.room.id, comm::equip_from_room(ctx.avatar.label.as_str(), item.label.as_str()));
        },

        Err(EquipError::ItemNotProvide) | Err(EquipError::Fail) => {
            outputs.private(player_id, comm::equip_what());
        },

        Err(EquipError::ItemNotEquipable { item_id }) => {
            let item = container.items.get(item_id);
            outputs.private(player_id, comm::equip_item_invalid(item.label.as_str()));
        },

        Err(EquipError::ItemNotFound { label }) => {
            outputs.private(player_id, comm::equip_item_not_found(label.as_str()));
        },
    }
}

pub enum EquipAction {
    Success {
        item_id: ItemId,
    }
}

pub enum EquipError {
    Fail,
    ItemNotProvide,
    ItemNotEquipable { item_id: ItemId },
    ItemNotFound { label: String },
}

pub fn do_equip(container: &mut Container, player_id: PlayerId, args: Vec<String>) -> Result<EquipAction, EquipError> {
    let item_label = match args.get(1) {
        Some(str) => str,
        None => {
            return Err(EquipError::ItemNotProvide);
        }
    };

    let ctx = container.get_player_context(player_id);
    let inventory = container.items.get_inventory_list(ItemLocation::Mob { mob_id: ctx.avatar.id });

    // TODO: move to a util search with more powerful capabilities
    let item = inventory.iter()
        .find(|item| item.label.eq_ignore_ascii_case(item_label));

    match item {
        Some(item) if item.can_equip() => {
            let mob_id = ctx.avatar.id;
            let item_id = item.id;

            container.items.equip(ItemLocation::Mob { mob_id }, item_id)
                .map_err(|_| EquipError::Fail)?;

            Ok(EquipAction::Success { item_id })
        },

        Some(item) => {
            Err(EquipError::ItemNotEquipable { item_id: item.id })
        },

        None => {
            Err(EquipError::ItemNotFound { label: item_label.to_string() })
        }
    }
}

#[test]
pub fn test1() {
    assert!(false);
}
