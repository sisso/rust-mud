use super::comm;
use super::controller::Outputs;
use super::domain::*;
use super::container::Container;
use super::mob::*;
use super::player::*;

pub fn look(container: &mut Container, outputs: &mut Outputs, player_id: &PlayerId) {
    let ctx = container.get_player_context(&player_id);

    outputs.private(
        player_id.clone(),
        comm::look_description(container, &ctx)
    );
}

pub fn say(container: &mut Container, outputs: &mut Outputs, player_id: &PlayerId, msg: String) {
    let ctx = container.get_player_context(player_id);
    let player_msg = comm::say_you_say(&msg);
    let room_msg = comm::say_someone_said(&ctx.avatar.label, &msg);

    outputs.private(player_id.clone(), player_msg);
    outputs.room(player_id.clone(), ctx.avatar.room_id, room_msg);
}

pub fn mv(container: &mut Container, outputs: &mut Outputs, player_id: &PlayerId, dir: Dir) {
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
            let ctx = container.get_player_context(&player_id);

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

pub fn kill(container: &mut Container, outputs: &mut Outputs, player_id: &PlayerId, target: &MobId) {
    let ctx = container.get_player_context(player_id);
    let target_mob = container.mobs.get(&target);

    let player_msg = comm::kill_player_attack(target_mob);
    let room_msg = comm::kill_mob_attack_someone(&ctx.avatar, &target_mob);

    let avatar_id = ctx.avatar.id.clone();
    let room_id = ctx.room.id;

    container.mobs.set_mob_kill_target(&avatar_id, target);

    outputs.private(player_id.clone(), player_msg);
    outputs.room(player_id.clone(), room_id, room_msg);
}
