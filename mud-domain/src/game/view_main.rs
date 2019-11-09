use super::Outputs;
use super::domain::*;

use super::actions;
use super::comm;
use super::container::Container;
use crate::game::{actions_admin, input_handle_items, mob, inventory};
use commons::PlayerId;
use std::collections::HashSet;

pub fn handle(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, input: &str) {
    match input {
        "h" | "help" => {
            outputs.private(player_id, comm::help());
        },

        "l" | "look" => {
            actions::look(container, outputs, player_id);
        },

        "n" | "s" | "e" | "w" => {
            let dir = match input.as_ref() {
                "n" => Dir::N,
                "s" => Dir::S,
                "e" => Dir::E,
                "w" => Dir::W,
                _ => panic!("invalid input {}", input),
            };

            actions::mv(container, outputs, player_id, dir);
        },

        "uptime" => {
            outputs.private(player_id, comm::uptime(container.time.total));
        },

        "rest" => {
            actions::rest(container, outputs, player_id);
        },

        "stand" => {
            actions::stand(container, outputs, player_id);
        },

        "stats" | "inv" | "score" => {
            let ctx = container.get_player_context(player_id);
            let item_inventory = inventory::get_inventory_list(&container.locations, &container.items, ctx.mob.id);
            let equiped = container.equips.get(ctx.mob.id).unwrap_or(HashSet::new());
            outputs.private(player_id, comm::stats(&ctx.mob, &item_inventory, &equiped));
        },

        _ if has_command(input, &["pick"]) || has_command(&input, &["get"]) => {
            let _ = input_handle_items::pickup(container, outputs, player_id, parse_arguments(input));
        },

        _ if has_command(input, &["drop"]) => {
            input_handle_items::drop(container, outputs, player_id, parse_arguments(input))
        },

        _ if has_command(input, &["remove"]) => {
            input_handle_items::strip(container, outputs, player_id, parse_arguments(input))
        },

        _ if has_command(input, &["equip"]) => {
            input_handle_items::equip(container, outputs, player_id, parse_arguments(input))
        },

        _ if has_command(input, &["examine "]) => {
            action_examine(container, outputs,player_id, input);
        },

        _ if has_command(input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
            let ctx = container.get_player_context(player_id);
            let mobs = mob::search_mobs_at(&container.mobs, &container.locations, ctx.room.id, target);
            let candidate = mobs.first().map(|i| i.id);

            match candidate {
                Some(mob_id) if !container.mobs.is_avatar(&mob_id) => {
                    let _ = actions::attack(container, outputs, player_id, mob_id);
                },
                Some(_) => {
                    outputs.private(player_id, comm::kill_can_not_kill_players(&target));
                },
                None => {
                    outputs.private(player_id, comm::kill_target_not_found(&target));
                }
            }
        },

        _ if input.starts_with("say ")  => {
            let msg = input["say ".len()..].to_string();
            actions::say(container, outputs, player_id, msg);
        },

        _ if input.starts_with("admin ")  => {
            let arguments = parse_arguments(input);
            if arguments.len() != 2 {
                outputs.private(player_id, comm::admin_invalid_command());
                return;
            }

            match arguments.get(1).unwrap().as_ref() {
                "suicide" => {
                    let pctx = container.get_player_context(player_id);
                    outputs.private(player_id, comm::admin_suicide());
                    outputs.room(player_id, pctx.room.id, comm::admin_suicide_others(pctx.mob.label.as_ref()));

                    let mob_id = pctx.mob.id;
                    actions_admin::kill(container, outputs, mob_id);
                },
                other => {
                    outputs.private(player_id, comm::admin_invalid_command());
                }
            }
        },

        _ => {
            outputs.private(player_id, comm::unknown_input(input));
        },
    }
}

fn action_examine(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, input: &str) {
    let target = parse_command(input, &["examine "]);
    let ctx = container.get_player_context(player_id);
    let mobs = mob::search_mobs_at(&container.mobs, &container.locations, ctx.room.id, target);

    match mobs.first() {
        Some(mob) => {
            let item_location = mob.id;
            let mob_inventory = inventory::get_inventory_list(&container.locations, &container.items, mob.id);
            let equiped = container.equips.get(item_location).unwrap_or(HashSet::new());
            outputs.private(player_id, comm::examine_target(mob, &mob_inventory, &equiped));
            return;
        },
        _ => {},
    }

    let items = inventory::search(&container.locations, &container.items, ctx.room.id, target);
    match items.first() {
        Some(item) => {
            let item_inventory = inventory::get_inventory_list(&container.locations, &container.items, item.id);
            outputs.private(player_id, comm::examine_target_item(item, &item_inventory));
            return;
        },
        _ => {},
    }

    // else
    outputs.private(player_id, comm::examine_target_not_found(target));
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

fn parse_arguments(input: &str) -> Vec<&str> {
    input
        .split_ascii_whitespace()
        .into_iter()
        .collect()
}
