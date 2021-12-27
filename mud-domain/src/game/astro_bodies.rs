use crate::errors::Error::NotFoundFailure;
use crate::errors::{AsResult, Error, Result};
use crate::game::container::Container;
use crate::game::location::Locations;
use crate::game::ships::ShipId;
use commons::{DeltaTime, ObjId};
use logs::*;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum AstroBodyKind {
    Star,
    Planet,
    Moon,
    JumpGate,
    Ship,
    AsteroidField,
    Station,
}

impl AstroBodyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AstroBodyKind::Star => "star",
            AstroBodyKind::Planet => "planet",
            AstroBodyKind::Moon => "moon",
            AstroBodyKind::JumpGate => "jump_gate",
            AstroBodyKind::Ship => "ship",
            AstroBodyKind::AsteroidField => "asteroid_field",
            AstroBodyKind::Station => "station",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AstroBodies {
    index: HashMap<AstroBodyId, AstroBody>,
}

impl AstroBodies {
    pub fn new() -> Self {
        AstroBodies {
            index: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, value: AstroBody) -> Option<AstroBody> {
        info!("{:?} upsert {:?}", value.id, value);
        self.index.insert(value.id, value)
    }

    pub fn remove(&mut self, id: AstroBodyId) -> Option<AstroBody> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: AstroBodyId) -> Option<&AstroBody> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: AstroBodyId) -> Option<&mut AstroBody> {
        self.index.get_mut(&id)
    }

    pub fn update_orbit(&mut self, id: AstroBodyId, orbital_distance: f32) -> Result<()> {
        self.index.get_mut(&id).as_result()?.orbit_distance = orbital_distance;
        Ok(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}

pub struct TravelPlan {
    /// root body where positions are computed
    pub root_body_id: ObjId,
    /// parent body in common between current and target orbit
    pub reference_body_id: ObjId,
    /// global distance from current orbit
    pub from_distance: DistanceMkm,
    /// global distance to target orbit
    pub to_distance: DistanceMkm,
    /// total travel time from current orbit to common reference body, then from common
    /// bodie to target orbit
    pub total_distance: DistanceMkm,
}

pub fn travel_plan(
    locations: &Locations,
    astro_bodies: &AstroBodies,
    ship_id: ObjId,
    target_id: ObjId,
) -> Result<TravelPlan> {
    let ship_parents: Vec<&AstroBody> = locations
        .list_parents_inclusive(ship_id)
        .into_iter()
        .flat_map(|id| astro_bodies.get(id))
        .collect();

    let target_parents: Vec<&AstroBody> = locations
        .list_parents_inclusive(target_id)
        .into_iter()
        .flat_map(|id| astro_bodies.get(id))
        .collect();

    let root_body_id = ship_parents.last().unwrap().id;

    // find common reference body
    let (reference_ship_index, reference_target_index) = ship_parents
        .iter()
        .enumerate()
        .flat_map(|(index, one)| {
            if let Some(pos) = target_parents.iter().position(|other| other.id == one.id) {
                Some((index, pos))
            } else {
                None
            }
        })
        .next()
        .as_result()?;

    let reference_body_id = ship_parents[reference_ship_index].id;
    assert_eq!(reference_body_id, target_parents[reference_target_index].id);

    // trace!("root body: {:?}", root_body_id);
    // trace!("reference body: {:?}", reference_body_id);
    // trace!("ship parents {:?}", ship_parents);
    // trace!("target parents {:?}", target_parents);

    // compute global distance
    let from_distance: f32 = ship_parents
        .iter()
        .map(|astro| astro.orbit_distance)
        .sum::<f32>();

    let to_distance: f32 = target_parents
        .iter()
        .map(|astro| astro.orbit_distance)
        .sum::<f32>();

    // trace!("root ship index {:?}", reference_ship_index);
    // trace!("root target index {:?}", reference_target_index);
    // trace!("ship parents {:?}", ship_parents);
    // trace!("target parents {:?}", target_parents);
    // trace!("from {:?}", from_distance);
    // trace!("to {:?}", to_distance);

    // compute reference body diff to transfer between bodies
    let reference_distance_ship = ship_parents[reference_ship_index - 1].orbit_distance;
    let reference_distance_target = if reference_target_index == 0 {
        0.0
    } else {
        target_parents[reference_target_index - 1].orbit_distance
    };

    let root_distance = (reference_distance_ship - reference_distance_target).abs();

    // compute distance from reference body
    let ship_to_root_distance: f32 = ship_parents[0..reference_ship_index - 1]
        .iter()
        .map(|astro| astro.orbit_distance)
        .sum();

    let target_to_root_distance: f32 = if reference_target_index == 0 {
        0.0
    } else {
        target_parents[0..reference_target_index - 1]
            .iter()
            .map(|astro| astro.orbit_distance)
            .sum()
    };

    // trace!("root distance {:?}", root_distance);
    // trace!("ship_to_root_distance {:?}", ship_to_root_distance);
    // trace!("target_to_root_distance {:?}", target_to_root_distance);
    // trace!("ship_distance {:?}", ship_distance);
    // trace!("target_distance {:?}", target_distance);

    let total_distance = root_distance + ship_to_root_distance + target_to_root_distance;

    Ok(TravelPlan {
        root_body_id,
        reference_body_id,
        from_distance,
        to_distance,
        total_distance,
    })
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

#[cfg(test)]
mod test {
    use crate::game::astro_bodies::{travel_plan, AstroBodies, AstroBody, AstroBodyKind};
    use crate::game::location::Locations;
    use crate::game::triggers::Event::Obj;

    ///     0 - star (0)
    ///         1 - planet 1 (10)
    ///             2 - moon 1 (2)
    ///                 3 - ship (1)
    ///             6 - moon 2 (4)
    ///         4 - planet 2 (20)
    ///             5 - station (2)
    fn scenery_1(locations: &mut Locations, astros: &mut AstroBodies) {
        locations.set(1.into(), 0.into());
        locations.set(2.into(), 1.into());
        locations.set(3.into(), 2.into());
        locations.set(6.into(), 1.into());
        locations.set(4.into(), 0.into());
        locations.set(5.into(), 4.into());
        astros.upsert(AstroBody::new(0.into(), 0.0, AstroBodyKind::Star));

        astros.upsert(AstroBody::new(1.into(), 10.0, AstroBodyKind::Planet));

        astros.upsert(AstroBody::new(2.into(), 2.0, AstroBodyKind::Moon));

        astros.upsert(AstroBody::new(3.into(), 1.0, AstroBodyKind::Ship));

        astros.upsert(AstroBody::new(4.into(), 20.0, AstroBodyKind::Planet));

        astros.upsert(AstroBody::new(5.into(), 2.0, AstroBodyKind::Station));

        astros.upsert(AstroBody::new(6.into(), 4.0, AstroBodyKind::Moon));
    }

    /// 0 - star (0)
    ///     1 - planet 1 (10)
    ///         2 - ship (1)
    ///     3 - planet 2 (20)
    fn scenery_2(locations: &mut Locations, astros: &mut AstroBodies) {
        locations.set(1.into(), 0.into());
        locations.set(2.into(), 1.into());
        locations.set(3.into(), 0.into());
        astros.upsert(AstroBody::new(0.into(), 0.0, AstroBodyKind::Star));

        astros.upsert(AstroBody::new(1.into(), 10.0, AstroBodyKind::Planet));

        astros.upsert(AstroBody::new(2.into(), 1.0, AstroBodyKind::Ship));

        astros.upsert(AstroBody::new(3.into(), 20.0, AstroBodyKind::Planet));
    }

    #[test]
    fn test_travel_plan() {
        let mut locations = Locations::new();
        let mut astros = AstroBodies::new();

        scenery_1(&mut locations, &mut astros);

        {
            let plan = travel_plan(&locations, &astros, 3.into(), 5.into()).unwrap();
            assert_eq!(plan.root_body_id.as_u32(), 0);
            assert_eq!(plan.reference_body_id.as_u32(), 0);
            assert_eq!(plan.from_distance as u32, 13);
            assert_eq!(plan.to_distance as u32, 22);
            assert_eq!(plan.total_distance as u32, (2 + 1) + (20 - 10) + 2);
        }

        {
            let plan = travel_plan(&locations, &astros, 3.into(), 6.into()).unwrap();
            assert_eq!(plan.root_body_id.as_u32(), 0);
            assert_eq!(plan.reference_body_id.as_u32(), 1);
            assert_eq!(plan.from_distance as u32, 13);
            assert_eq!(plan.to_distance as u32, 14);
            assert_eq!(plan.total_distance as u32, 1 + (4 - 2));
        }
    }

    #[test]
    fn test_travel_plan_basic_system() {
        let mut locations = Locations::new();
        let mut astros = AstroBodies::new();

        scenery_2(&mut locations, &mut astros);

        let plan = travel_plan(&locations, &astros, 2.into(), 3.into()).unwrap();
        assert_eq!(plan.root_body_id.as_u32(), 0);
        assert_eq!(plan.reference_body_id.as_u32(), 0);
        assert_eq!(plan.from_distance as u32, 11);
        assert_eq!(plan.to_distance as u32, 20);
        assert_eq!(plan.total_distance as u32, 1 + (20 - 10));
    }

    #[test]
    fn test_travel_plan_around_start() {
        let mut locations = Locations::new();
        let mut astros = AstroBodies::new();

        scenery_2(&mut locations, &mut astros);
        // put ship in orbit of main body
        locations.set(2.into(), 0.into());
        astros
            .upsert(AstroBody::new(2.into(), 14.0, AstroBodyKind::Ship))
            .unwrap();

        // compute travel to planet 1
        let plan = travel_plan(&locations, &astros, 2.into(), 1.into()).unwrap();
        assert_eq!(plan.root_body_id.as_u32(), 0);
        assert_eq!(plan.reference_body_id.as_u32(), 0);
        assert_eq!(plan.from_distance as u32, 14);
        assert_eq!(plan.to_distance as u32, 10);
        assert_eq!(plan.total_distance as u32, 14 - 10);

        // compute travel to planet 2
        let plan = travel_plan(&locations, &astros, 2.into(), 3.into()).unwrap();
        assert_eq!(plan.root_body_id.as_u32(), 0);
        assert_eq!(plan.reference_body_id.as_u32(), 0);
        assert_eq!(plan.from_distance as u32, 14);
        assert_eq!(plan.to_distance as u32, 20);
        assert_eq!(plan.total_distance as u32, 20 - 14);
    }

    #[test]
    fn test_travel_with_sector_parent() {
        let mut locations = Locations::new();
        let mut astros = AstroBodies::new();

        scenery_2(&mut locations, &mut astros);
        locations.set(0.into(), 99.into());

        let plan = travel_plan(&locations, &astros, 2.into(), 3.into()).unwrap();
        assert_eq!(plan.reference_body_id.as_u32(), 0);
        assert_eq!(plan.from_distance as u32, 11);
        assert_eq!(plan.to_distance as u32, 20);
        assert_eq!(plan.total_distance as u32, 1 + (20 - 10));
    }

    #[test]
    fn test_travel_from_station_into_orbiting_moon() {
        let mut locations = Locations::new();
        let mut astros = AstroBodies::new();

        scenery_1(&mut locations, &mut astros);
        locations.set(0.into(), 99.into());

        let plan = travel_plan(&locations, &astros, 3.into(), 1.into()).unwrap();
        assert_eq!(plan.root_body_id.as_u32(), 0);
        assert_eq!(plan.reference_body_id.as_u32(), 1);
        assert_eq!(plan.from_distance as u32, 13);
        assert_eq!(plan.to_distance as u32, 10);
        assert_eq!(plan.total_distance as u32, 1 + 2);
    }
}
