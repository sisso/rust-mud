use super::Outputs;
use super::domain::*;
use super::player::PlayerId;

use super::actions;
use super::actions_items;
use super::comm;
use super::item::ItemLocation;
use super::container::Container;
use crate::game::actions_admin;

pub fn handle(time: &GameTime, container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, input: &str) {
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

            actions::mv(container, outputs, player_id, dir)
        },

        "uptime" => {
            outputs.private(player_id, comm::uptime(time.total));
        },

        "rest" => {
            actions::rest(time, container, outputs, player_id);
        },

        "stand" => {
            actions::stand(time, container, outputs, player_id);
        },

        "stats" => {
            let ctx = container.get_player_context(player_id);
            let location = ItemLocation::Mob { mob_id: ctx.avatar.id };
            let item_inventory = container.items.get_inventory_list(location);
            let equiped = container.items.get_equiped(location);
            outputs.private(player_id, comm::stats(&ctx.avatar, &item_inventory, &equiped));
        },

        _ if has_command(input, &["pick"]) || has_command(&input, &["get"]) => {
            actions_items::pickup(container, outputs, player_id, parse_arguments(input))
        },

        _ if has_command(input, &["equip"]) => {
            actions_items::equip(container, outputs, player_id, parse_arguments(input))
        },

        _ if has_command(input, &["examine "]) => {
            action_examine(container, outputs,player_id, input);
        },

        _ if has_command(input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
            let ctx = container.get_player_context(player_id);
            let mobs = container.mobs.search(Some(&ctx.avatar.room_id), Some(&target));
            let candidate = mobs.first().map(|i| i.id);

            match candidate {
                Some(mob_id) if !container.mobs.is_avatar(&mob_id) => {
                    actions::attack(container, outputs, player_id, mob_id);
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
                    outputs.room(player_id, pctx.room.id, comm::admin_suicide_others(pctx.avatar.label.as_ref()));

                    let mob_id = pctx.avatar.id;
                    actions_admin::kill(time, container, outputs, mob_id);
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
    let mobs = container.mobs.search(Some(&ctx.avatar.room_id), Some(&target));

    match mobs.first() {
        Some(mob) => {
            let item_location = ItemLocation::Mob { mob_id: mob.id };
            let mob_inventory = container.items.get_inventory_list(item_location);
            let equiped = container.items.get_equiped(item_location);
            outputs.private(player_id, comm::examine_target(mob, &mob_inventory, &equiped));
            return;
        },
        _ => {},
    }

    let items = container.items.search(&ctx.avatar.room_id, &target);
    match items.first() {
        Some(item) => {
            let item_inventory = container.items.get_inventory_list(ItemLocation::Item { item_id: item.id });
            outputs.private(player_id, comm::examine_target_item(item, &item_inventory));
            return;
        },
        _ => {},
    }

    // else
    outputs.private(player_id, comm::examine_target_not_found(&target));
}

fn has_command(input: &str, commands: &[&str]) -> bool {
    for c in commands {
        if input.starts_with(c) {
            return true;
        }
    }

    return false;
}

fn parse_command(input: &str, commands: &[&str]) -> String {
    for c in commands {
        if input.starts_with(c) {
            return input[c.len()..].to_string();
        }
    }

    panic!("unable to parse!");
}

fn parse_arguments(input: &str) -> Vec<String> {
    input
        .split_ascii_whitespace()
        .into_iter()
        .map(|i| i.to_string())
        .collect()
}
