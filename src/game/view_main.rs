use crate::game::controller::Outputs;
use crate::game::domain::*;
use crate::game::player::PlayerId;

use super::actions;
use super::comm;

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

        _ if has_command(&input, &["k ", "kill "]) => {
            let target = parse_command(input, &["k ", "kill "]);
            let ctx = container.get_player_context(player_id);
            let mobs = container.search_mob_by_name_at(&ctx.avatar.room_id, &target);
            let candidate = mobs.first().map(|i| i.id);

            match candidate {
                Some(mob_id) => {
                    actions::kill(container, outputs, player_id, &mob_id);
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
