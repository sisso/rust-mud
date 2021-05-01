use crate::game::comm::ShowSectorTreeBodyKind::BodyKind;
use crate::random_grid::{LevelGrid, RandomGridCfg};
use commons::V2I;
use rand::rngs::StdRng;
use rand::{Rng, RngCore};
use rand_distr::{Binomial, Distribution, Normal};

#[derive(Clone, Debug)]
pub enum RDistrib {
    MinMax(f32, f32),
    Normal(f32, f32),
}

impl RDistrib {
    pub fn next(&self, rng: &mut StdRng) -> f32 {
        match self {
            RDistrib::MinMax(min, max) => rng.gen_range(min..max),

            RDistrib::Normal(mean, std_dev) => {
                let normal = Normal::new(mean, std_dev).unwrap();
                normal.sample(rng) as f32
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AstroProb {
    pub count_prob: RDistrib,
    pub distance_prob: RDistrib,
}

#[derive(Clone, Debug)]
pub struct UniverseCfg {
    pub planets_prob: AstroProb,
    pub moons_prob: AstroProb,
    pub asteroids_prob: AstroProb,
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
pub enum SpaceBodyKind {
    Star,
    Planet,
    Moon,
    AsteroidField,
}

#[derive(Clone, Debug)]
pub struct SpaceBody {
    pub index: usize,
    pub parent: usize,
    pub kind: SpaceBodyKind,
    pub distance: f32,
}

pub struct GenerateParams {
    pub sectors: RandomGridCfg,
}

#[derive(Debug)]
pub enum GenerateError {
    Generic(String),
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
    let star = SpaceBody {
        index: 0,
        parent: 0,
        kind: SpaceBodyKind::Star,
        distance: 0.0,
    };

    let mut bodies = vec![];

    let num_bodies = cfg.planets_prob.count_prob.next(rng);
    let num_bodies = next_incremental_prob(rng, cfg.system_body_prob, 1.0);
    for index in 1..(num_bodies + 1) {
        let body = new_body(cfg, rng, index as usize, 0);
        bodies.extend(body);
    }

    System {
        coords,
        bodies: bodies,
    }
}

fn new_body(cfg: &UniverseCfg, rng: &mut StdRng, index: usize, parent: usize) -> Vec<SpaceBody> {
    let distance = rng.gen_range(cfg.min_distance..cfg.max_distance);
    let kind = if rng.gen_bool(0.75) {
        SpaceBodyKind::Planet
    } else {
        SpaceBodyKind::AsteroidField
    };

    vec![SpaceBody {
        index,
        parent,
        kind,
        distance,
    }]
}

fn next_incremental_prob(rng: &mut StdRng, mut prob: f32, decay: f32) -> u32 {
    let mut v = 0;

    loop {
        if rng.gen_bool(prob as f64) {
            v += 1;
            prob *= decay;
        } else {
            return v;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::SeedableRng;
    use rand_distr::{Binomial, Distribution, Exp, Normal};

    // #[test]
    // fn test_incremental_prob() {
    //     let mut rng = StdRng::seed_from_u64(1);
    //
    //     for _ in 0..10 {
    //         println!("{}", next_incremental_prob(&mut rng, 0.85, 0.9));
    //     }
    // }

    #[test]
    fn test_incremental_prob_2() {
        let mut rng = StdRng::seed_from_u64(0);

        println!();
        for _ in 0..1000 {
            let normal = Normal::new(4.0, 2.0).unwrap();
            let v: f32 = normal.sample(&mut rng);
            let v: i32 = v.round().max(0.0) as i32;
            print!("{}\t", v);
        }
        println!();

        for _ in 0..10 {
            let bin = Binomial::new(20, 0.3).unwrap();
            let v = bin.sample(&mut rng);
            println!("{} is from a binomial distribution", v);
        }
    }

    #[test]
    fn test() {
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);

        let cfg = UniverseCfg {
            planets_prob: AstroProb {
                count_prob: RDistrib::Normal(4.0, 2.0),
                distance_prob: RDistrib::Normal(1.0, 0.5),
            },
            moons_prob: AstroProb {
                count_prob: RDistrib::Normal(2.0, 1.0),
                distance_prob: RDistrib::Normal(0.1, 0.01),
            },
            asteroids_prob: AstroProb {
                count_prob: RDistrib::Normal(1.0, 1.0),
                distance_prob: RDistrib::Normal(1.5, 0.5),
            },
        };

        let params = GenerateParams {
            sectors: RandomGridCfg {
                width: 4,
                height: 4,
                portal_prob: 0.25,
                deep_levels: 0,
            },
        };

        println!("{:?}", generate(&cfg, &params, &mut rng).unwrap());
    }
}
