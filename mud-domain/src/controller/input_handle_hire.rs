use crate::errors::Error::NotFoundFailure;
use crate::errors::{Error, Result};
use crate::game::actions::out;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::{comm, Outputs};
use crate::utils::strinput::StrInput;
use logs::*;

pub fn hire(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        warn!("{:?} player has no location", mob_id);
        outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let candidates = container
        .locations
        .list_at(location_id)
        .filter(|obj_id| container.hires.exist(*obj_id))
        .collect::<Vec<_>>();

    let args = input.plain_arguments();
    let founds = container.labels.search_codes(&candidates, args);

    if let Some(&hired_id) = founds.first() {
        crate::game::actions_hire::hire(container, outputs, mob_id, hired_id)
    } else {
        let labels = container.labels.resolve_labels(&candidates);

        if args.is_empty() {
            outputs.private(mob_id, comm::hire_list(labels));
            Ok(())
        } else {
            outputs.private(mob_id, comm::hire_fail_not_found(args));
            outputs.private(mob_id, comm::hire_list(labels));
            Err(NotFoundFailure)
        }
    }
}
