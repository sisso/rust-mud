use crate::game::comm::ShowSectorTreeBodyKind::BodyKind;
use crate::random_grid::{LevelGrid, RandomGridCfg};
use crate::utils::prob::{self, RDistrib, Weighted};
use commons::V2I;
use rand::rngs::StdRng;
use rand::{Rng, RngCore};
use rand_distr::{Binomial, Distribution, Normal};

#[derive(Clone, Debug)]
pub struct AstroProb {
    pub count_prob: RDistrib,
    pub distance_prob: RDistrib,
}

#[derive(Clone, Debug)]
pub struct UniverseCfg {
    pub planets_prob: AstroProb,
    pub moons_prob: AstroProb,
    pub moons_moons_prob: AstroProb,
    pub asteroids_prob: AstroProb,
    pub biomes_kinds: Vec<Weighted<String>>,
    pub atm_kinds: Vec<Weighted<String>>,
    pub ocean_kinds: Vec<Weighted<String>>,
    pub gravity_force: RDistrib,
    pub planet_size: RDistrib,
    pub star_kinds: Vec<Weighted<String>>,
}

#[derive(Clone, Debug)]
pub struct Universe {
    pub systems: Vec<System>,
}

#[derive(Clone, Debug)]
pub struct System {
    pub coords: V2I,
    pub bodies: Vec<SpaceBody>,
}

#[derive(Clone, Debug)]
pub enum BodyDesc {
    Star {
        kind: String,
    },
    AsteroidField {},
    Planet {
        atmosphere: String,
        gravity: f32,
        biome: String,
        ocean: String,
        size: f32,
    },
}

#[derive(Clone, Debug)]
pub struct SpaceBody {
    pub index: usize,
    pub parent: usize,
    pub distance: f32,
    pub desc: BodyDesc,
}

pub struct GenerateParams {
    pub sectors: RandomGridCfg,
}

#[derive(Debug)]
pub enum GenerateError {
    Generic(String),
}

pub struct PlanetSubCfg {
    max_distance: f32,
}

pub struct IdGen {
    pub v: usize,
}

impl IdGen {
    pub fn next(&mut self) -> usize {
        let v = self.v;
        self.v += 1;
        v
    }
}

pub fn generate(
    cfg: &UniverseCfg,
    params: &GenerateParams,
    rng: &mut StdRng,
) -> Result<Universe, GenerateError> {
    let mut systems = vec![];

    let level = LevelGrid::new(&params.sectors, rng);
    for x in 0..level.width {
        for y in 0..level.height {
            let system = new_system(cfg, rng, V2I::new(x as i32, y as i32));
            systems.push(system);
        }
    }

    Ok(Universe { systems })
}

fn new_system(cfg: &UniverseCfg, rng: &mut StdRng, coords: V2I) -> System {
    let mut igen = IdGen { v: 0 };

    let star_kind = prob::select(rng, &cfg.star_kinds).unwrap();

    let star_i = igen.next();
    let star = SpaceBody {
        index: star_i,
        parent: star_i,
        distance: 0.0,
        desc: BodyDesc::Star {
            kind: star_kind.clone(),
        },
    };

    let mut bodies = vec![star];

    let num_bodies = cfg.planets_prob.count_prob.next_int(rng);
    for i in 0..num_bodies {
        let body = new_planet(cfg, rng, &mut igen, star_i);
        bodies.extend(body);
    }

    System {
        coords,
        bodies: bodies,
    }
}

fn new_planet(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    igen: &mut IdGen,
    parent: usize,
) -> Vec<SpaceBody> {
    let mut distance = cfg.planets_prob.distance_prob.next(rng);

    let planet_i = igen.next();
    let atm = prob::select(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select(rng, &cfg.biomes_kinds).unwrap().clone();
    let ocean = prob::select(rng, &cfg.ocean_kinds).unwrap().clone();

    let planet = SpaceBody {
        index: planet_i,
        parent,
        distance,
        desc: BodyDesc::Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            size: cfg.planet_size.next(rng),
        },
    };

    let mut bodies = vec![planet];

    let num_m = cfg.moons_prob.count_prob.next_int(rng);

    for i in 0..num_m {
        let body = new_moon(
            cfg,
            &PlanetSubCfg {
                max_distance: distance * 0.75,
            },
            rng,
            igen,
            planet_i,
        );
        bodies.extend(body);
    }

    bodies
}

fn new_moon(
    cfg: &UniverseCfg,
    sub_cfg: &PlanetSubCfg,
    rng: &mut StdRng,
    igen: &mut IdGen,
    parent: usize,
) -> Vec<SpaceBody> {
    let mut distance;
    loop {
        distance = cfg.planets_prob.distance_prob.next(rng);
        if distance <= sub_cfg.max_distance {
            break;
        }
    }

    let planet_i = igen.next();
    let atm = prob::select(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select(rng, &cfg.biomes_kinds).unwrap().clone();
    let ocean = prob::select(rng, &cfg.ocean_kinds).unwrap().clone();

    let moon = SpaceBody {
        index: planet_i,
        parent,
        distance,
        desc: BodyDesc::Planet {
            atmosphere: atm,
            gravity: cfg.gravity_force.next(rng),
            biome: biome,
            ocean: ocean,
            size: cfg.planet_size.next(rng),
        },
    };

    vec![moon]
}

#[cfg(test)]
mod test {}
