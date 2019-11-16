use crate::game::crafts::{CraftId, CraftCommand};
use commons::{ObjId, PlayerId};
use crate::game::{Outputs, comm};
use crate::game::container::Container;

pub fn move_to(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, craft_id: CraftId, target_id: ObjId) -> Result<(),()> {
    container.crafts.set_command(craft_id, CraftCommand::MoveTo { target_id })
        .map(|ok| {
            outputs.private(player_id, comm::space_move());
            ok
        })
        .map_err(|err| {
            outputs.private(player_id, comm::space_move_invalid());
            err
        })
}
