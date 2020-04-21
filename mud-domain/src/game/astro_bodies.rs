use crate::errors::{Error, Result};
use crate::game::location::Locations;
use commons::ObjId;
use logs::*;
use std::collections::HashMap;

pub type AstroBodyId = ObjId;

/// orbit distance in 1000 * km
// TODO: replace by distance struct
pub type DistanceMkm = f32;

pub fn km_to_mkm(value: f32) -> f32 {
    value / 1000.0
}

//#[derive(Clone, Copy, PartialEq, Eq, Debug)]
//pub struct Distance(f32);

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum AstroBodyKind {
    Star,
    Planet,
    Moon,
    JumpGate,
    Ship,
    AsteroidField,
    Station,
}

#[derive(Clone, Debug)]
pub struct AstroBody {
    pub id: AstroBodyId,
    pub orbit_distance: DistanceMkm,
    pub kind: AstroBodyKind,
    pub jump_target_id: Option<ObjId>,
}

impl AstroBody {
    pub fn new(id: AstroBodyId, orbit_distance: DistanceMkm, kind: AstroBodyKind) -> Self {
        AstroBody {
            id,
            orbit_distance,
            kind,
            jump_target_id: None,
        }
    }

    pub fn get_low_orbit(&self) -> DistanceMkm {
        km_to_mkm(100.0)
    }
}

#[derive(Clone, Debug)]
pub struct AstroBodies {
    index: HashMap<AstroBodyId, AstroBody>,
}

impl AstroBodies {
    pub fn new() -> Self {
        AstroBodies {
            index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: AstroBody) -> Result<()> {
        if self.index.contains_key(&value.id) {
            return Err(Error::ConflictException);
        }

        info!("{:?} insert {:?}", value.id, value);
        self.index.insert(value.id, value);
        Ok(())
    }

    pub fn update(&mut self, value: AstroBody) -> Result<()> {
        if !self.index.contains_key(&value.id) {
            return Err(Error::ConflictException);
        }

        info!("{:?} update {:?}", value.id, value);
        self.index.insert(value.id, value);
        Ok(())
    }

    pub fn remove(&mut self, id: AstroBodyId) -> Option<AstroBody> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: AstroBodyId) -> Option<&AstroBody> {
        self.index.get(&id)
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}

pub fn find_start_of(
    locations: &Locations,
    astro_bodies: &AstroBodies,
    obj_id: ObjId,
) -> Option<ObjId> {
    locations
        .list_parents(obj_id)
        .into_iter()
        .flat_map(|astro| astro_bodies.get(astro))
        .find(|astro| astro.kind == AstroBodyKind::Star)
        .map(|astro| astro.id)
}
