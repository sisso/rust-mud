use super::domain::*;
use super::Outputs;

use super::actions;
use super::comm;
use super::container::Container;
use crate::game::comm::InventoryDesc;
use crate::game::{actions_admin, input_handle_items, input_handle_space, inventory, mob};
use commons::{ObjId, PlayerId, UResult, UERR, UOK};
use std::collections::HashSet;

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
    player_id: PlayerId,
    input: &str,
) -> UResult {
    let mob_id = container.players.get_mob(player_id)?;

    match input {
        "h" | "help" => {
            outputs.private(player_id, comm::help());
            UOK
        }

        "l" | "look" => {
            actions::look(container, outputs, player_id);
            UOK
        }

        "n" | "s" | "e" | "w" | "enter" | "out" | "exit" => {
            let dir = match input.as_ref() {
                "n" => Dir::N,
                "s" => Dir::S,
                "e" => Dir::E,
                "w" => Dir::W,
                "enter" => Dir::Enter,
                "out" | "exit" => Dir::Out,
                _ => panic!("invalid input {}", input),
            };

            actions::mv(container, outputs, player_id, dir)
        }

        "uptime" => {
            outputs.private(player_id, comm::uptime(container.time.total));
            UOK
        }

        "rest" => actions::rest(container, outputs, player_id),

        "stand" => actions::stand(container, outputs, player_id),

        "stats" | "inv" | "score" => {
            let ctx = container.get_player_context(player_id);
            outputs.private(
                player_id,
                comm::stats(
                    &ctx.mob.attributes,
                    &inventory_to_desc(container, ctx.player.mob_id),
                ),
            );
            UOK
        }

        _ if has_command(input, &["pick"]) || has_command(&input, &["get"]) => {
            input_handle_items::pickup(container, outputs, player_id, parse_arguments(input))
        }

        _ if has_command(input, &["drop"]) => {
            input_handle_items::drop(container, outputs, player_id, parse_arguments(input))
        }

        _ if has_command(input, &["remove"]) => {
            input_handle_items::strip(container, outputs, player_id, parse_arguments(input))
        }

        _ if has_command(input, &["equip"]) => {
            input_handle_items::equip(container, outputs, player_id, parse_arguments(input))
        }

        _ if has_command(input, &["examine "]) => {
            action_examine(container, outputs, player_id, input)
        }

        _ if has_command(input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
            let ctx = container.get_player_context(player_id);
            let mobs = mob::search_mobs_at(
                &container.labels,
                &container.locations,
                &container.mobs,
                ctx.room.id,
                target,
            );
            let candidate = mobs.first();

            match candidate {
                Some(mob_id) if !container.mobs.is_avatar(*mob_id) => {
                    actions::attack(container, outputs, player_id, *mob_id);
                    UOK
                }
                Some(_) => {
                    outputs.private(player_id, comm::kill_can_not_kill_players(&target));
                    UERR
                }
                None => {
                    outputs.private(player_id, comm::kill_target_not_found(&target));
                    UERR
                }
            }
        }

        _ if input.starts_with("say ") => {
            let msg = input["say ".len()..].to_string();
            actions::say(container, outputs, Some(player_id), mob_id, msg)
        }

        _ if input.starts_with("admin ") => {
            let arguments = parse_arguments(input);
            if arguments.len() != 2 {
                outputs.private(player_id, comm::admin_invalid_command());
                return UERR;
            }

            match arguments.get(1).unwrap().as_ref() {
                "suicide" => {
                    let pctx = container.get_player_context(player_id);
                    let mob_id = pctx.mob.id;
                    let mob_label = container.labels.get_label_f(mob_id);
                    outputs.private(player_id, comm::admin_suicide());
                    outputs.room(
                        player_id,
                        pctx.room.id,
                        comm::admin_suicide_others(mob_label),
                    );
                    actions_admin::kill(container, outputs, mob_id)
                }
                _other => {
                    outputs.private(player_id, comm::admin_invalid_command());
                    UERR
                }
            }
        }

        "sm" | "map" => input_handle_space::show_starmap(container, outputs, player_id, mob_id),

        "move" => input_handle_space::move_list_targets(container, outputs, player_id, mob_id),

        _ if input.starts_with("move ") => input_handle_space::move_to(
            container,
            outputs,
            player_id,
            mob_id,
            parse_arguments(input),
        ),

        "land" => input_handle_space::land_list(container, outputs, player_id, mob_id),

        _ if input.starts_with("land ") => input_handle_space::land_at(
            container,
            outputs,
            player_id,
            mob_id,
            parse_arguments(input),
        ),

        "launch" => input_handle_space::launch(container, outputs, player_id, mob_id),

        _ => {
            outputs.private(player_id, comm::unknown_input(input));
            UERR
        }
    }
}

fn action_examine(
    container: &Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
    input: &str,
) -> UResult {
    let target_label = parse_command(input, &["examine "]);
    let ctx = container.get_player_context(player_id);
    let mobs = mob::search_mobs_at(
        &container.labels,
        &container.locations,
        &container.mobs,
        ctx.room.id,
        target_label,
    );

    match mobs.first().cloned() {
        Some(mob_id) => {
            let mob_label = container.labels.get_label_f(mob_id);
            let mob = container.mobs.get(mob_id).unwrap();
            outputs.private(
                player_id,
                comm::examine_target(
                    mob_label,
                    &mob.attributes,
                    &inventory_to_desc(container, mob_id),
                ),
            );
            return UOK;
        }
        _ => {}
    }

    let items = inventory::search(
        &container.labels,
        &container.locations,
        &container.items,
        ctx.room.id,
        target_label,
    );
    match items.first().cloned() {
        Some(item_id) => {
            let item_label = container.labels.get_label_f(item_id);
            outputs.private(
                player_id,
                comm::examine_target_item(item_label, &inventory_to_desc(container, item_id)),
            );
            return UOK;
        }
        _ => {}
    }

    // else
    outputs.private(player_id, comm::examine_target_not_found(target_label));
    UERR
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
