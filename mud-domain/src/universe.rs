use crate::game::comm::ShowSectorTreeBodyKind::BodyKind;
use crate::random_grid::{LevelGrid, RandomGridCfg};
use crate::utils::prob::RDistrib;
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

    let star_i = igen.next();
    let star = SpaceBody {
        index: star_i,
        parent: star_i,
        kind: SpaceBodyKind::Star,
        distance: 0.0,
    };

    let mut bodies = vec![star];

    let num_bodies = cfg.planets_prob.count_prob.next_int(rng);
    for i in 0..num_bodies {
        let body = new_planet(
            cfg,
            &PlanetSubCfg {
                max_distance: 100.0,
            },
            rng,
            &mut igen,
            star_i,
            0,
        );
        bodies.extend(body);
    }

    System {
        coords,
        bodies: bodies,
    }
}

fn new_planet(
    cfg: &UniverseCfg,
    sub_cfg: &PlanetSubCfg,
    rng: &mut StdRng,
    igen: &mut IdGen,
    parent: usize,
    deep: usize,
) -> Vec<SpaceBody> {
    let mut distance = 0.0;

    loop {
        distance = if deep == 0 {
            cfg.planets_prob.distance_prob.next(rng)
        } else if deep == 1 {
            cfg.moons_prob.distance_prob.next(rng)
        } else {
            cfg.moons_moons_prob.distance_prob.next(rng)
        };

        if distance < sub_cfg.max_distance {
            break;
        }
    }

    let kind = SpaceBodyKind::Planet;

    let planet_i = igen.next();
    let planet = SpaceBody {
        index: planet_i,
        parent,
        kind,
        distance,
    };

    let mut bodies = vec![planet];

    let num_m = match deep {
        0 => cfg.moons_prob.count_prob.next_int(rng),
        1 => cfg.moons_moons_prob.count_prob.next_int(rng),
        _ => 0,
    };

    // println!(
    //     "planet {} parent {} on deep {} children {}",
    //     planet_i, parent, deep, num_m
    // );

    for i in 0..num_m {
        let body = new_planet(
            cfg,
            &PlanetSubCfg {
                max_distance: distance * 0.75,
            },
            rng,
            igen,
            planet_i,
            deep + 1,
        );
        bodies.extend(body);
    }

    bodies
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
    use commons::tree::Tree;
    use itertools::Itertools;
    use rand::{thread_rng, SeedableRng};
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
        // let mut rng = StdRng::seed_from_u64(0);
        let mut rng = thread_rng();

        println!();
        let mut all = vec![];
        for i in 0..100 {
            let d = rand_distr::ChiSquared::new(3.0).unwrap();
            let v: f32 = d.sample(&mut rng);
            let v: i32 = v.round().max(0.0) as i32;
            print!("{} ", v);
            if i % 10 == 0 {
                println!()
            }

            all.push(v);
        }
        all.sort();
        let grouped = all.iter().group_by(|i| **i);
        for g in &grouped {
            println!("{}: {}", g.0, g.1.count());
        }

        println!();

        // println!();
        // for _ in 0..100 {
        //     let normal = Normal::new(4.0, 2.0).unwrap();
        //     let v: f32 = normal.sample(&mut rng);
        //     let v: i32 = v.round().max(0.0) as i32;
        //     print!("{}\t", v);
        // }
        // println!();
        //
        // for _ in 0..10 {
        //     let bin = Binomial::new(20, 0.3).unwrap();
        //     let v = bin.sample(&mut rng);
        //     println!("{} is from a binomial distribution", v);
        // }
    }

    #[test]
    fn test() {
        let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);

        let cfg = UniverseCfg {
            planets_prob: AstroProb {
                count_prob: RDistrib::List {
                    values: vec![0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 5.0, 6.0],
                },
                distance_prob: RDistrib::MinMax(0.5, 10.0),
            },
            moons_prob: AstroProb {
                count_prob: RDistrib::List {
                    values: vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0],
                },
                distance_prob: RDistrib::MinMax(0.25, 1.0),
            },
            moons_moons_prob: AstroProb {
                count_prob: RDistrib::List {
                    values: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0],
                },
                distance_prob: RDistrib::MinMax(0.01, 0.25),
            },
            asteroids_prob: AstroProb {
                count_prob: RDistrib::List {
                    values: vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0],
                },
                distance_prob: RDistrib::MinMax(1.0, 20.0),
            },
        };

        let params = GenerateParams {
            sectors: RandomGridCfg {
                width: 2,
                height: 2,
                portal_prob: 0.25,
                deep_levels: 0,
            },
        };

        let universe = generate(&cfg, &params, &mut rng).unwrap();

        for (i, b) in universe.systems.iter().enumerate() {
            // let mut tree = Tree::new();
            // for b in b.bodies.iter() {
            //     tree.insert(b.index, b.parent);
            // }
            //
            println!("System {:?}", b.coords);

            for i in b.bodies.iter() {
                println!("- {:?}", i);
            }

            // let star = &b.bodies[0];
            // // println!("- {:?}", star);
            // for i in tree.children(star.index) {
            //     println!("- {:?}", &b.bodies[i]);
            // }
        }

        println!("{:?}", universe);
    }
}
