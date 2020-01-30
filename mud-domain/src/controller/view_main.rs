use std::collections::HashSet;

use commons::{ObjId, PlayerId};

use crate::errors::{AsResult, Error, Result};
use crate::game::actions;
use crate::game::comm;
use crate::game::comm::InventoryDesc;
use crate::game::container::{Container};
use crate::game::domain::Dir;
use crate::game::mob::MobId;
use crate::game::Outputs;
use crate::game::{actions_admin, inventory, mob};
use crate::utils::strinput::StrInput;
use logs::*;
use super::{input_handle_items, input_handle_space, input_handle_vendors};

fn inventory_to_desc(container: &Container, obj_id: ObjId) -> Vec<InventoryDesc> {
    let equip = container.equips.get(obj_id);

    inventory::get_inventory_list(&container.locations, &container.items, obj_id)
        .into_iter()
        .map(|item| {
            let item_label = container.labels.get_label_f(item.id);

            InventoryDesc {
                id: item.id,
                label: item_label,
                amount: item.amount,
                equipped: equip.contains(&item.id),
            }
        })
        .collect()
}

// TODO: normalize use o ctx
pub fn handle(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: &str,
) -> Result<()> {
    let input = StrInput(input);

    // TODO: replace by first(), if a input want to be unique should check if there is no args
    match input.as_str() {
        "h" | "help" => {
            outputs.private(mob_id, comm::help());
            Ok(())
        }

        "l" | "look" => actions::look(container, outputs, mob_id),

        "n" => actions::mv(container, outputs, mob_id, Dir::N),

        "s" => actions::mv(container, outputs, mob_id, Dir::S),

        "e" => actions::mv(container, outputs, mob_id, Dir::E),

        "w" => actions::mv(container, outputs, mob_id, Dir::W),

        "enter" => actions::enter(container, outputs, mob_id, ""),

        _ if input.has_commands(&["enter"]) => {
            let args = input.plain_arguments();
            actions::enter(container, outputs, mob_id, args)
        }

        "exit" | "out" => actions::out(container, outputs, mob_id),

        "uptime" => {
            outputs.private(mob_id, comm::uptime(container.time.total));
            Ok(())
        }

        "rest" => actions::rest(container, outputs, mob_id),

        "stand" => actions::stand(container, outputs, mob_id),

        "stats" | "inv" | "score" => {
            let ctx = container.get_mob_ctx(mob_id).as_result()?;
            outputs.private(
                mob_id,
                comm::stats(
                    ctx.mob.xp,
                    &ctx.mob.attributes,
                    &inventory_to_desc(container, ctx.mob.id),
                ),
            );
            Ok(())
        }

        _ if input.has_commands(&["pick", "get"]) => {
            input_handle_items::pickup(container, outputs, mob_id, input)
        }

        _ if input.has_commands(&["drop"]) => {
            input_handle_items::drop(container, outputs, mob_id, input)
        }

        _ if input.has_commands(&["remove"]) => {
            input_handle_items::strip(container, outputs, mob_id, input)
        }

        _ if input.has_commands(&["equip"]) => {
            input_handle_items::equip(container, outputs, mob_id, input)
        }

        _ if input.has_commands(&["examine"]) => {
            action_examine(container, outputs, mob_id, input.plain_arguments())
        }

        _ if input.has_commands(&["k", "kill"]) => {
            let target = input.plain_arguments();

            let ctx = container.get_mob_ctx(mob_id).as_result()?;
            let mobs = mob::search_mobs_at(
                &container.labels,
                &container.locations,
                &container.mobs,
                ctx.room.id,
                target,
            );

            let candidate = mobs.first();
            match candidate {
                Some(&target_mob_id) if !container.mobs.is_avatar(target_mob_id) => {
                    let _ = actions::attack(container, outputs, mob_id, target_mob_id);
                    Ok(())
                }
                Some(_) => {
                    outputs.private(mob_id, comm::kill_can_not_kill_players(&target));
                    Err(Error::InvalidArgumentFailure)
                }
                None => {
                    outputs.private(mob_id, comm::kill_target_not_found(&target));
                    Err(Error::InvalidArgumentFailure)
                }
            }
        }

        _ if input.has_command("say") => {
            let msg = input.plain_arguments();
            actions::say(container, outputs, mob_id, msg)
        }

        _ if input.has_command("admin") => {
            let arguments = input.split();
            if arguments.len() != 2 {
                outputs.private(mob_id, comm::admin_invalid_command());
                return Err(Error::InvalidArgumentFailure);
            }

            match arguments.get(1).unwrap().as_ref() {
                "suicide" => {
                    let pctx = container.get_mob_ctx(mob_id).as_result()?;
                    let target_mob_id = pctx.mob.id;
                    let mob_label = container.labels.get_label_f(target_mob_id);
                    outputs.private(mob_id, comm::admin_suicide());
                    outputs.broadcast(None, pctx.room.id, comm::admin_suicide_others(mob_label));
                    actions_admin::kill(container, outputs, target_mob_id)
                }
                _ => {
                    outputs.private(mob_id, comm::admin_invalid_command());
                    Err(Error::InvalidArgumentFailure)
                }
            }
        }

        "sm" | "map" => input_handle_space::show_startree(container, outputs, mob_id),

        "move" => input_handle_space::move_list_targets(container, outputs, mob_id),

        _ if input.has_command("move") => {
            input_handle_space::move_to(container, outputs, mob_id, &input)
        }

        "land" => input_handle_space::land_list(container, outputs, mob_id),

        _ if input.has_command("land") => {
            input_handle_space::land_at(container, outputs, mob_id, input.split())
        }

        "launch" => input_handle_space::launch(container, outputs, mob_id),

        _ if input.has_command("list") => {
            input_handle_vendors::list(container, outputs, mob_id, input)
        }

        _ if input.has_command("buy") => {
            input_handle_vendors::buy(container, outputs, mob_id, input)
        }

        _ if input.has_command("sell") => {
            input_handle_vendors::sell(container, outputs, mob_id, input)
        }

        _ => {
            outputs.private(mob_id, comm::unknown_input(input.as_str()));
            Err(Error::InvalidArgumentFailure)
        }
    }
}

fn action_examine(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    target_label: &str,
) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let mobs = mob::search_mobs_at(
        &container.labels,
        &container.locations,
        &container.mobs,
        room_id,
        target_label,
    );

    match mobs.first().cloned() {
        Some(target_id) => {
            let mob_label = container.labels.get_label_f(target_id);
            let target_mob = container.mobs.get(target_id).unwrap();
            outputs.private(
                mob_id,
                comm::examine_target(
                    mob_label,
                    target_mob.xp,
                    &target_mob.attributes,
                    &inventory_to_desc(container, target_id),
                ),
            );
            return Ok(());
        }
        _ => {}
    }

    let items = inventory::search(
        &container.labels,
        &container.locations,
        &container.items,
        room_id,
        target_label,
    );
    match items.first().cloned() {
        Some(item_id) => {
            let item_label = container.labels.get_label_f(item_id);
            outputs.private(
                mob_id,
                comm::examine_target_item(item_label, &inventory_to_desc(container, item_id)),
            );
            return Ok(());
        }
        _ => {}
    }

    // else
    outputs.private(mob_id, comm::examine_target_not_found(target_label));
    Err(Error::InvalidArgumentFailure)
}
