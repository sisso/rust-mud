use super::controller::Outputs;
use super::domain::*;
use super::player::PlayerId;

use super::actions;
use super::comm;
use super::container::Container;

pub fn handle(container: &mut Container, outputs: &mut Outputs, player_id: &PlayerId, input: String) {
    match input.as_ref() {
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
            outputs.private(player_id.clone(), comm::uptime(&container.get_time()));
        },

        "stats" => {
            let ctx = container.get_player_context(player_id);
            outputs.private(player_id.clone(), comm::stats(&ctx.avatar));
        },

        _ if has_command(&input, &["examine "]) => {
            let target = parse_command(input, &["examine "]);
            let ctx = container.get_player_context(player_id);
            let mobs = container.mobs.search(Some(&ctx.avatar.room_id), Some(&target));
            match mobs.first() {
                Some(mob) => {
                    let mob_inventory = container.items.get_mobs_inventory_list(&mob.id);
                    outputs.private(player_id.clone(), comm::examine_target(mob, &mob_inventory));
                },
                None => {
                    outputs.private(player_id.clone(), comm::examine_target_not_found(&target));
                },
            }

            let items = container.items.search(&ctx.avatar.room_id, &target);
            match items.first() {
                Some(item) => {
                    let item_inventory = container.items.get_item_inventory_list(&item.id);
                    outputs.private(player_id.clone(), comm::examine_target_item(item, &item_inventory));
                },
                None => {
                    outputs.private(player_id.clone(), comm::examine_target_not_found(&target));
                },
            }
        },

        _ if has_command(&input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
            let ctx = container.get_player_context(player_id);
            let mobs = container.mobs.search(Some(&ctx.avatar.room_id), Some(&target));
            let candidate = mobs.first().map(|i| i.id);

            match candidate {
                Some(mob_id) if !container.mobs.is_avatar(&mob_id) => {
                    actions::kill(container, outputs, player_id, &mob_id);
                },
                Some(_) => {
                    outputs.private(player_id.clone(), comm::kill_can_not_kill_players(&target));
                },
                None => {
                    outputs.private(player_id.clone(), comm::kill_target_not_found(&target));
                }
            }
        },

        _ if input.starts_with("say ")  => {
            let msg = input["say ".len()..].to_string();
            actions::say(container, outputs, player_id, msg);
        },

        _ => {
            outputs.private(*player_id, comm::unknown_input(input));
        },
    }
}

fn has_command(input: &String, commands: &[&str]) -> bool {
    for c in commands {
        if input.starts_with(c) {
            return true;
        }
    }

    return false;
}

fn parse_command(input: String, commands: &[&str]) -> String {
    for c in commands {
        if input.starts_with(c) {
            return input[c.len()..].to_string();
        }
    }

    panic!("unable to parse!");
}
