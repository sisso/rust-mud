use super::comm;
use super::controller::Outputs;
use super::domain::*;
use super::container::Container;
use super::mob::*;
use super::player::*;

pub fn look(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let ctx = container.get_player_context(player_id);

    outputs.private(
        player_id.clone(),
        comm::look_description(container, &ctx)
    );
}

pub fn say(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, msg: String) {
    let ctx = container.get_player_context(player_id);
    let player_msg = comm::say_you_say(&msg);
    let room_msg = comm::say_someone_said(&ctx.avatar.label, &msg);

    outputs.private(player_id.clone(), player_msg);
    outputs.room(player_id.clone(), ctx.avatar.room_id, room_msg);
}

pub fn mv(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, dir: Dir) {
    let ctx = container.get_player_context(player_id);
    let player_id = player_id.clone();

    let exit_room_id = ctx.room.get_exit(&dir);

    match exit_room_id {
        Some(exit_room_id) => {
            let previous_room_id = ctx.avatar.room_id;

            // change entity in place
            let mut mob = ctx.avatar.clone();
            mob.room_id = exit_room_id;
            container.mobs.update(mob);

            // get new player ctx
            let ctx = container.get_player_context(player_id);

            let look = comm::look_description(&container, &ctx);

            let player_msg = format!("{}\n\n{}", comm::move_you_move(&dir), look);
            let enter_room_msg = comm::move_come(&ctx.avatar.label, &dir.inv());
            let exit_room_msg = comm::move_goes(&ctx.avatar.label, &dir);

            outputs.private(player_id, player_msg);
            outputs.room(player_id, previous_room_id, exit_room_msg);
            outputs.room(player_id, ctx.room.id, enter_room_msg);
        },
        None => {
            outputs.private(player_id, comm::move_not_possible(&dir));
        }
    }
}

pub fn attack(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, target: &MobId) {
    let ctx = container.get_player_context(player_id);
    let target_mob = container.mobs.get(&target);

    let player_msg = comm::attack_player_initiate(target_mob);
    let room_msg = comm::attack_mob_initiate_attack(&ctx.avatar, &target_mob);

    let avatar_id = ctx.avatar.id;
    let room_id = ctx.room.id;

    container.mobs.set_mob_attack_target(avatar_id, target);

    outputs.private(player_id, player_msg);
    outputs.room(player_id, room_id, room_msg);
}

pub fn rest(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let player = container.players.get_player_by_id(player_id);
    let mob = container.mobs.get(&player.avatar_id);
    let mob_id = mob.id;

    if mob.is_combat() {
        outputs.private(player_id, comm::rest_fail_in_combat());
        return;
    }

    outputs.private(player_id, comm::rest_start());
    outputs.room(player_id, mob.room_id,comm::rest_start_others(mob.label.as_str()));

    container.mobs.set_state_resting(mob_id, true);
}

pub fn stand(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let player = container.players.get_player_by_id(player_id);
    let mob = container.mobs.get(&player.avatar_id);
    let mob_id = mob.id;

    if mob.is_resting() {
        outputs.private(player_id, comm::stand_fail_not_resting());
        return;
    }

    outputs.private(player_id, comm::stand_up());
    outputs.room(player_id, mob.room_id,comm::stand_up_others(mob.label.as_str()));

    container.mobs.set_state_resting(mob_id, false);
}
