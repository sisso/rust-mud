use std::collections::HashSet;

use commons::{ObjId, PlayerId};

use crate::errors::{AsResult, Error, Result};
use crate::game::{actions_admin, input_handle_items, input_handle_space, inventory, mob};
use crate::game::comm::InventoryDesc;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::actions;
use crate::game::comm;
use crate::game::Outputs;
use crate::game::domain::Dir;

fn inventory_to_desc(container: &Container, obj_id: ObjId) -> Vec<InventoryDesc> {
    let equip = container.equips.get(obj_id).unwrap_or(HashSet::new());
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

pub fn handle(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: &str,
) -> Result<()> {
    match input {
        "h" | "help" => {
            outputs.private(mob_id, comm::help());
            Ok(())
        }

        "l" | "look" => {
            actions::look(container, outputs, mob_id)
        }

        "n" | "s" | "e" | "w" => {
            let dir = match input.as_ref() {
                "n" => Dir::N,
                "s" => Dir::S,
                "e" => Dir::E,
                "w" => Dir::W,
                _ => panic!("invalid input {}", input),
            };

            actions::mv(container, outputs, mob_id, dir)
        }

        "enter" => {
            actions::enter(container, outputs, mob_id, "")
        }

        _ if has_command(input, &["enter "]) => {
            let args= parse_command(input, &["enter "]);
            actions::enter(container, outputs, mob_id, args)
        }

        "exit" | "out" => {
            actions::out(container, outputs, mob_id)
        }

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
                    &ctx.mob.attributes,
                    &inventory_to_desc(container, ctx.mob.id),
                ),
            );
            Ok(())
        }

        _ if has_command(input, &["pick ", "get "])  => {
            input_handle_items::pickup(container, outputs, mob_id, parse_arguments(input))
        }

        _ if has_command(input, &["drop "]) => {
            input_handle_items::drop(container, outputs, mob_id, parse_arguments(input))
        }

        _ if has_command(input, &["remove "]) => {
            input_handle_items::strip(container, outputs, mob_id, parse_arguments(input))
        }

        _ if has_command(input, &["equip "]) => {
            input_handle_items::equip(container, outputs, mob_id, parse_arguments(input))
        }

        _ if has_command(input, &["examine "]) => {
            action_examine(container, outputs, mob_id, input)
        }

        _ if has_command(input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
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
                    Err(Error::IllegalArgument)
                }
                None => {
                    outputs.private(mob_id, comm::kill_target_not_found(&target));
                    Err(Error::IllegalArgument)
                }
            }
        }

        _ if input.starts_with("say ") => {
            let msg = input["say ".len()..].to_string();
            actions::say(container, outputs, mob_id, msg)
        }

        _ if input.starts_with("admin ") => {
            let arguments = parse_arguments(input);
            if arguments.len() != 2 {
                outputs.private(mob_id, comm::admin_invalid_command());
                return Err(Error::IllegalArgument);
            }

            match arguments.get(1).unwrap().as_ref() {
                "suicide" => {
                    let pctx = container.get_mob_ctx(mob_id).as_result()?;
                    let target_mob_id = pctx.mob.id;
                    let mob_label = container.labels.get_label_f(target_mob_id);
                    outputs.private(mob_id, comm::admin_suicide());
                    outputs.broadcast(
                        None,
                        pctx.room.id,
                        comm::admin_suicide_others(mob_label),
                    );
                    actions_admin::kill(container, outputs, target_mob_id)
                }
                _other => {
                    outputs.private(mob_id, comm::admin_invalid_command());
                    Err(Error::IllegalArgument)
                }
            }
        }

        "sm" | "map" => input_handle_space::show_startree(container, outputs, mob_id),

        "move" => input_handle_space::move_list_targets(container, outputs, mob_id),

        _ if input.starts_with("move ") => input_handle_space::move_to(
            container,
            outputs,
            mob_id,
            parse_arguments(input),
        ),

        "land" => input_handle_space::land_list(container, outputs, mob_id),

        _ if input.starts_with("land ") => input_handle_space::land_at(
            container,
            outputs,
            mob_id,
            parse_arguments(input),
        ),

        "launch" => input_handle_space::launch(container, outputs, mob_id),

        _ => {
            outputs.private(mob_id, comm::unknown_input(input));
            Err(Error::IllegalArgument)
        }
    }
}

fn action_examine(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: &str,
) -> Result<()> {
    let target_label = parse_command(input, &["examine "]);
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
            let mob = container.mobs.get(target_id).unwrap();
            outputs.private(
                mob_id,
                comm::examine_target(
                    mob_label,
                    &mob.attributes,
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
    Err(Error::IllegalArgument)
}

fn has_command(input: &str, commands: &[&str]) -> bool {
    for c in commands {
        if input.starts_with(c) {
            return true;
        }
    }

    return false;
}

fn parse_command<'a>(input: &'a str, commands: &[&str]) -> &'a str {
    for c in commands {
        if input.starts_with(c) {
            return &input[c.len()..];
        }
    }

    panic!("unable to parse!");
}

// TODO: drop first argument
fn parse_arguments(input: &str) -> Vec<&str> {
    input.split_ascii_whitespace().into_iter().collect()
}
