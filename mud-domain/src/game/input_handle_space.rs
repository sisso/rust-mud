use commons::{PlayerId};
use crate::game::container::Container;
use crate::game::{actions_craft};
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::game::space_utils::*;

pub fn show_starmap(container: &Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, craft_location);
    outputs.private(player_id, comm::space_show_sectormap(&objects));
    Ok(())
}

pub fn move_list_targets(container: & Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, craft_location);
    outputs.private(player_id, comm::space_show_move_targets(&objects));
    Ok(())
}

pub fn move_to(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId, input: Vec<&str>) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    input.get(1).ok_or(()).and_then(|label| {
        find_surface_target(container, craft_location, label)
    }).and_then(|target_id| {
        actions_craft::move_to(container, outputs, player_id, craft_id, target_id)
    }).map_err(|_| {
        outputs.private(player_id, comm::space_move_invalid());
    })
}

