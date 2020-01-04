use crate::game::actions_craft;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::space_utils::*;
use crate::game::{comm, Outputs};
use crate::utils::text;
use commons::PlayerId;

use crate::errors::{AsResult, Error, Result};
use crate::game::actions_craft::{do_land_at, do_launch};

pub fn show_startree(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    let (ship_id, sector_id) = get_craft_and_sector(container, outputs, mob_id)?;
    let bodies = find_astro_bodies(container, sector_id);
    outputs.private(mob_id, comm::show_sectortree(&bodies));
    Ok(())
}

#[deprecated]
pub fn show_starmap(container: &Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let (craft_id, sector_id) = get_craft_and_sector(container, outputs, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, sector_id);
    outputs.private(mob_id, comm::show_surface_map(&objects));
    Ok(())
}

pub fn move_list_targets(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    let (craft_id, craft_location) = get_craft_and_sector(container, outputs, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, craft_location);
    outputs.private(mob_id, comm::space_show_move_targets(&objects));
    Ok(())
}

pub fn move_to(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: Vec<&str>,
) -> Result<()> {
    let (craft_id, craft_location) = get_craft_and_sector(container, outputs, mob_id)?;
    input
        .get(1)
        .ok_or(Error::InvalidArgumentFailure)
        .and_then(|label| find_surface_target(container, craft_location, label))
        .and_then(|target_id| {
            actions_craft::move_to(container, outputs, mob_id, craft_id, target_id)
        })
        .map_err(|_| {
            outputs.private(mob_id, comm::space_move_invalid());
            Error::InvalidArgumentFailure
        })
}

pub fn land_list(container: &Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    get_craft(container, mob_id)
        .map(|craft_id| {
            let labels = search_near_landing_sites(container, craft_id)
                .into_iter()
                .map(|id| container.labels.get_label_f(id))
                .collect();

            outputs.private(mob_id, comm::space_land_list(&labels));
        })
        .ok_or_else(|| {
            outputs.private(mob_id, comm::space_land_invalid());
            Error::InvalidArgumentFailure
        })
}

pub fn land_at(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: Vec<&str>,
) -> Result<()> {
    let result = match (input.get(1), get_craft(container, mob_id)) {
        (Some(input), Some(craft_id)) => {
            let sites = search_near_landing_sites(container, craft_id);
            let labels = container.labels.resolve_codes(&sites);

            match text::search_label(input, &labels).first().cloned() {
                Some(index) => {
                    let landing_room = sites[index];
                    do_land_at(container, outputs, craft_id, landing_room)
                }
                None => Err(Error::InvalidArgumentFailure),
            }
        }
        _ => Err(Error::InvalidArgumentFailure),
    };

    result.map_err(|e| {
        outputs.private(mob_id, comm::space_land_invalid());
        e
    })
}

pub fn launch(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    get_craft(container, mob_id)
        .as_result()
        .map_err(|e| {
            outputs.private(mob_id, comm::space_invalid_not_in_craft());
            e
        })
        .and_then(|craft_id| do_launch(container, outputs, mob_id, craft_id))
}
