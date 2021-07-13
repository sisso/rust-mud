use crate::game::comm::ShowSectorTreeBodyKind::BodyKind;
use crate::random_grid::{LevelGrid, RandomGridCfg};
use crate::utils::prob::{self, select, RDistrib, Weighted};
use commons::V2I;
use rand::prelude::*;
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
    pub resources: Vec<Resource>,
    pub system_resources_max: u32,
    pub system_resources_amount: RDistrib,
}

#[derive(Clone, Debug)]
pub struct Universe {
    pub systems: Vec<System>,
    pub portals: Vec<(usize, usize)>,
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
    AsteroidField {
        resources: Vec<BodyResource>,
    },
    Planet {
        atmosphere: String,
        gravity: f32,
        biome: String,
        ocean: String,
        size: f32,
        resources: Vec<BodyResource>,
    },
    JumpGate {},
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

pub struct SystemSubCfg {
    num_gates: u32,
    coords: V2I,
}

pub struct PlanetSubCfg {
    max_distance: f32,
}

#[derive(Debug, Clone)]
pub struct BodyResource {
    resource: String,
    amount: f32,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub kind: String,
    pub prob: f32,
    pub always: Vec<String>,
    pub require: Vec<String>,
    pub forbidden: Vec<String>,
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
    for y in 0..level.height {
        for x in 0..level.width {
            let system = new_system(
                cfg,
                rng,
                &SystemSubCfg {
                    num_gates: level.neighbors_connected(level.get_index(x, y)).len() as u32,
                    coords: V2I::new(x as i32, y as i32),
                },
            );
            systems.push(system);
        }
    }

    println!("{}", level.print());

    Ok(Universe {
        systems,
        portals: level.portals.iter().cloned().collect(),
    })
}

fn new_system(cfg: &UniverseCfg, rng: &mut StdRng, sub_cfg: &SystemSubCfg) -> System {
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
    for _ in 0..num_bodies {
        let body = new_planet(cfg, rng, &mut igen, star_i);
        bodies.extend(body);
    }

    for _ in 0..sub_cfg.num_gates {
        let jump_gate = SpaceBody {
            index: igen.next(),
            parent: star_i,
            distance: cfg.planets_prob.distance_prob.next(rng),
            desc: BodyDesc::JumpGate {},
        };
        bodies.push(jump_gate);
    }

    System {
        coords: sub_cfg.coords.clone(),
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

    let resources = generate_resources(&cfg, rng, &atm, &biome, &ocean);

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
            resources: resources,
        },
    };

    let mut bodies = vec![planet];

    let num_m = cfg.moons_prob.count_prob.next_int(rng);

    for _ in 0..num_m {
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
    let mut distance = cfg.planets_prob.distance_prob.next(rng);
    for _ in 0..10 {
        if distance <= sub_cfg.max_distance {
            break;
        }
        distance = cfg.planets_prob.distance_prob.next(rng);
    }

    let planet_i = igen.next();
    let atm = prob::select(rng, &cfg.atm_kinds).unwrap().clone();
    let biome = prob::select(rng, &cfg.biomes_kinds).unwrap().clone();
    let ocean = prob::select(rng, &cfg.ocean_kinds).unwrap().clone();
    let resources = generate_resources(&cfg, rng, &atm, &biome, &ocean);

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
            resources,
        },
    };

    vec![moon]
}

fn generate_resources(
    cfg: &UniverseCfg,
    rng: &mut StdRng,
    _atm: &str,
    biome: &str,
    _ocean: &str,
) -> Vec<BodyResource> {
    fn cmp(a: &str, b: &str) -> bool {
        a.eq_ignore_ascii_case(b)
    }

    let mut rng: StdRng = SeedableRng::seed_from_u64(rng.gen());
    let rng = &mut rng;
    let mut resources = vec![];
    let mut candidates: Vec<Weighted<&Resource>> = vec![];

    cfg.resources
        .iter()
        .flat_map(|r| {
            if r.forbidden
                .iter()
                .find(|n| cmp(n.as_str(), biome))
                .is_some()
            {
                None
            } else if r.always.iter().find(|n| cmp(n.as_str(), biome)).is_some() {
                resources.push(BodyResource {
                    resource: r.kind.to_string(),
                    amount: 1.0,
                });
                None
            } else if !r.require.is_empty()
                && r.require.iter().find(|n| cmp(n.as_str(), biome)).is_none()
            {
                None
            } else {
                Some(r)
            }
        })
        .for_each(|r| {
            candidates.push(Weighted {
                prob: r.prob,
                value: r,
            })
        });

    for _ in resources.len()..(cfg.system_resources_max as usize) {
        let selected = commons::unwrap_or_continue!(select(rng, &candidates));
        if selected.kind == "none" {
            continue;
        }

        let amount = cfg.system_resources_amount.next_positive(rng);
        if amount <= 0.0 {
            continue;
        }

        match resources
            .iter_mut()
            .find(|i| i.resource.as_str() == selected.kind)
        {
            Some(found) => found.amount += cfg.system_resources_amount.next(rng),

            None => resources.push(BodyResource {
                resource: selected.kind.clone(),
                amount: amount,
            }),
        }
    }

    resources
}

#[cfg(test)]
mod test {}
