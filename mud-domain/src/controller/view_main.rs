use std::collections::HashSet;

use commons::{ObjId, PlayerId};

use super::{input_handle_items, input_handle_space, input_handle_vendors};
use crate::controller::{input_handle_hire, ViewHandleCtx};
use crate::errors::Error::NotFoundFailure;
use crate::errors::{AsResult, Error, Result};
use crate::game::actions;
use crate::game::comm;
use crate::game::comm::InventoryDesc;
use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::mob::MobId;
use crate::game::outputs::Outputs;
use crate::game::{actions_admin, inventory, mob};
use crate::utils::strinput::StrInput;
use logs::*;

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

pub fn handle(mut ctx: ViewHandleCtx, input: &str) -> Result<()> {
    let input = StrInput(input);

    // handle inputs per category
    let result = handle_general(&mut ctx, &input);
    match result {
        Ok(()) => return Ok(()),
        Err(NotFoundFailure) => {}
        Err(other) => {
            warn!("{:?} fail processing command {:?}", ctx.mob_id, other);
        }
    }

    let result = handle_ship(&mut ctx, &input);
    match result {
        Ok(()) => return Ok(()),
        Err(NotFoundFailure) => {}
        Err(other) => {
            warn!("{:?} fail processing command {:?}", ctx.mob_id, other);
        }
    }

    // handle legacy
    let (container, mob_id) = (ctx.container, ctx.mob_id);

    // TODO: replace by first(), if a input want to be unique should check if there is no args
    match input.as_str() {
        "l" | "look" => actions::look(container, mob_id),

        "n" => actions::mv(container, mob_id, Dir::N),

        "s" => actions::mv(container, mob_id, Dir::S),

        "e" => actions::mv(container, mob_id, Dir::E),

        "w" => actions::mv(container, mob_id, Dir::W),

        "u" => actions::mv(container, mob_id, Dir::U),

        "d" => actions::mv(container, mob_id, Dir::D),

        "enter" => actions::enter(container, mob_id, ""),

        _ if input.has_commands(&["enter"]) => {
            let args = input.plain_arguments();
            actions::enter(container, mob_id, args)
        }

        "exit" | "out" => actions::out(container, mob_id),

        "rest" => actions::rest(container, mob_id),

        "stand" => actions::stand(container, mob_id),

        "stats" | "inv" | "score" => {
            let ctx = container.get_mob_ctx(mob_id).as_result()?;
            container.outputs.private(
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
            input_handle_items::pickup(container, mob_id, input)
        }

        _ if input.has_commands(&["drop"]) => input_handle_items::drop(container, mob_id, input),

        _ if input.has_commands(&["remove"]) => input_handle_items::strip(container, mob_id, input),

        _ if input.has_commands(&["equip"]) => input_handle_items::equip(container, mob_id, input),

        _ if input.has_commands(&["examine"]) => {
            action_examine(container, mob_id, input.plain_arguments())
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
                    let _ = actions::attack(container, mob_id, target_mob_id);
                    Ok(())
                }
                Some(_) => {
                    container
                        .outputs
                        .private(mob_id, comm::kill_can_not_kill_players(&target));
                    Err(Error::InvalidArgumentFailure)
                }
                None => {
                    container
                        .outputs
                        .private(mob_id, comm::kill_target_not_found(&target));
                    Err(Error::InvalidArgumentFailure)
                }
            }
        }

        _ if input.has_command("say") => {
            let msg = input.plain_arguments();
            actions::say(container, mob_id, msg)
        }

        _ if input.has_command("admin") => {
            let arguments = input.split();
            if arguments.len() != 2 {
                container
                    .outputs
                    .private(mob_id, comm::admin_invalid_command());
                return Err(Error::InvalidArgumentFailure);
            }

            match arguments.get(1).unwrap().as_ref() {
                "suicide" => {
                    let pctx = container.get_mob_ctx(mob_id).as_result()?;
                    let target_mob_id = pctx.mob.id;
                    let room_id = pctx.room.id;
                    let mob_label = container.labels.get_label_f(target_mob_id);
                    container.outputs.private(mob_id, comm::admin_suicide());
                    container.outputs.broadcast(
                        None,
                        room_id,
                        comm::admin_suicide_others(mob_label),
                    );
                    actions_admin::force_kill(container, target_mob_id)
                }
                _ => {
                    container
                        .outputs
                        .private(mob_id, comm::admin_invalid_command());
                    Err(Error::InvalidArgumentFailure)
                }
            }
        }

        "sm" => input_handle_space::show_startree(container, mob_id),

        "map" => actions::show_map(container, mob_id),

        "move" => input_handle_space::move_list_targets(container, mob_id),

        _ if input.has_command("move") => input_handle_space::move_to(container, mob_id, &input),

        "land" => input_handle_space::land_list(container, mob_id),

        _ if input.has_command("land") => input_handle_space::land_at(container, mob_id, &input),

        "launch" => input_handle_space::launch(container, mob_id),

        _ if input.has_command("buy") => input_handle_vendors::buy(container, mob_id, input),

        _ if input.has_command("sell") => input_handle_vendors::sell(container, mob_id, input),

        _ if input.has_command("hire") => input_handle_hire::hire(container, mob_id, input),

        _ => {
            container
                .outputs
                .private(mob_id, comm::unknown_input(input.as_str()));
            Err(Error::InvalidArgumentFailure)
        }
    }
}

fn action_examine(container: &mut Container, mob_id: MobId, target_label: &str) -> Result<()> {
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
            container.outputs.private(
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
            container.outputs.private(
                mob_id,
                comm::examine_target_item(item_label, &inventory_to_desc(container, item_id)),
            );
            return Ok(());
        }
        _ => {}
    }

    // else
    container
        .outputs
        .private(mob_id, comm::examine_target_not_found(target_label));
    Err(Error::InvalidArgumentFailure)
}

pub fn handle_ship(ctx: &mut ViewHandleCtx, input: &StrInput) -> Result<()> {
    match input.first() {
        "jump" => {
            input_handle_space::jump(ctx);
            Ok(())
        }

        _ => Err(NotFoundFailure),
    }
}

pub fn handle_general(ctx: &mut ViewHandleCtx, input: &StrInput) -> Result<()> {
    match input.first() {
        "h" | "help" => {
            ctx.container.outputs.private(ctx.mob_id, comm::help());
            Ok(())
        }

        "uptime" => {
            ctx.container
                .outputs
                .private(ctx.mob_id, comm::uptime(ctx.container.time.total));
            Ok(())
        }

        _ => Err(NotFoundFailure),
    }
}
