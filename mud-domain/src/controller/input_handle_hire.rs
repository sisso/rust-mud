use crate::game::container::Container;
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::utils::strinput::StrInput;
use crate::errors::{Result, Error};
use logs::*;

pub fn hire(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {

    match input.parse_arguments() {
        args if args.is_empty() => show_hire_candidates(container, outputs, mob_id),
        _ => show_hire_candidates(container, outputs, mob_id),
    }
}

pub fn show_hire_candidates(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        warn!("{:?} player has no location", mob_id);
        outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let candidates = container.locations.list_at(location_id)
        .filter(|obj_id| container.hires.exist(*obj_id))
        .flat_map(|obj_id| {
            container.labels.get_label(obj_id)
        })
        .collect();

    outputs.private(mob_id, comm::hire_list(candidates));

    Ok(())
}
