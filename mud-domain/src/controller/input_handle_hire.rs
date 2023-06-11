use crate::errors::Error::NotFoundFailure;
use crate::errors::{Error, Result};
use crate::game::actions::out;
use crate::game::comm;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::utils::strinput::StrInput;

pub fn hire(container: &mut Container, mob_id: MobId, input: StrInput) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        log::warn!("{:?} player has no location", mob_id);
        container.outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let candidates = container
        .locations
        .list_at(location_id)
        .filter(|obj_id| container.hires.exist(*obj_id))
        .collect::<Vec<_>>();

    let args = input.plain_arguments();
    let labels = container.labels.resolve_labels(&candidates);

    if args.is_empty() {
        container.outputs.private(mob_id, comm::hire_list(labels));
        Ok(())
    } else {
        let founds = container.labels.search(&candidates, args);
        if let Some(&hired_id) = founds.first() {
            return crate::game::actions_hire::hire(container, mob_id, hired_id);
        }

        container
            .outputs
            .private(mob_id, comm::hire_fail_not_found(args));
        container.outputs.private(mob_id, comm::hire_list(labels));
        Err(NotFoundFailure)
    }
}
