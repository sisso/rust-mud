use crate::game::mob::MobId;
use crate::errors::*;
use crate::game::container::{Container, Ctx};
use crate::utils::strinput::StrInput;
use crate::game::{actions_vendor, Outputs, comm};
use commons::ObjId;

pub fn list(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, outputs, mob_id)?;
    actions_vendor::list(container, outputs, mob_id, vendor_id)
}

pub fn buy(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    unimplemented!();
}

pub fn sell(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    unimplemented!();
}

fn find_vendor_at_mob_location(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<ObjId> {
    let location_id = match container.locations.get(mob_id) {
        Some(location_id) => location_id,
        None => {
            outputs.private(mob_id, comm::vendor_operation_fail());
            return Err(Error::IllegalState);
        },
    };

    container.locations.list_at(location_id)
        .into_iter()
        .filter(|&id| container.vendors.exist(id))
        .next()
        .ok_or_else(|| {
            outputs.private(mob_id, comm::vendor_operation_fail());
            Error::NotFound
        })
}
