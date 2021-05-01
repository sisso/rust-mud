use commons::csv;
use mud_domain::random_grid::RandomGridCfg;
use mud_domain::universe::*;
use mud_domain::utils::prob::{self, RDistrib, Weighted};
use rand::prelude::StdRng;

fn main() {
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);

    let biomes = load_csv_into_weighted(
        r#"Arid	1
Desert	1
Barrent	1
Jungle	1
Swamp	1
Tropical	1
Ice	1
Water	1
Mountains	1
Gas	1"#,
    );

    let stars = load_csv_into_weighted(
        r#"Orange	2
Green	1
Yellow	2
Red	2
Blue	1
White	1"#,
    );

    let atmos = load_csv_into_weighted(
        r#"Breathable	1
Non-Breathable	2
Toxic	1
None	4"#,
    );

    let oceans = load_csv_into_weighted(
        r#"None	4
Salt water	2
Water	1
Acid	1
Amonia	1"#,
    );

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
        biomes_kinds: biomes,
        atm_kinds: atmos,
        ocean_kinds: oceans,
        gravity_force: RDistrib::MinMax(0.25, 10.0),
        planet_size: RDistrib::MinMax(0.1, 10.0),
        star_kinds: stars,
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

fn load_csv_into_weighted(raw: &str) -> Vec<Weighted<String>> {
    let mut r = vec![];
    let csv = csv::parse_csv_ext(raw, '\t');
    for row in &csv {
        r.push(Weighted {
            prob: row[1].parse().expect("fail to parse prob"),
            value: row[0].to_string(),
        });
    }
    r
}
