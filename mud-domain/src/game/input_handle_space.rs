use commons::{PlayerId, UERR};
use crate::game::container::Container;
use crate::game::{actions_craft};
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::game::space_utils::*;
use crate::utils::text;

use crate::game::actions_craft::do_land_at;

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

pub fn land_list(container: & Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    get_craft(container, mob_id).map(|craft_id| {
        let labels = search_near_landing_sites(container, craft_id)
            .into_iter().map(|id| {
                container.labels.get_label_f(id)
            }).collect();

        outputs.private(player_id, comm::space_land_list(&labels));
    }).map_err(|_| {
        outputs.private(player_id, comm::space_land_invalid());
    })
}

pub fn land_at(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId, input: Vec<&str>) -> Result<(),()> {
    let result = match (input.get(1), get_craft(container, mob_id)) {
        (Some(input), Ok(craft_id)) => {
            let sites = search_near_landing_sites(container, craft_id);
            let labels = container.labels.resolve_codes(&sites);

            match text::search_label(input, &labels).first().cloned() {
                Some(index) => {
                    let landing_room = sites[index];
                    do_land_at(container, outputs, craft_id, landing_room)
                },
                None => UERR,
            }
        },
        _ => UERR,
    };

    result.map_err(|_err| {
        outputs.private(player_id, comm::space_land_invalid())
    })
}

pub fn launch(container: & Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    UERR
}

