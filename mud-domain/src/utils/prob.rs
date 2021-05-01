use rand::rngs::StdRng;
use rand::{Rng, RngCore};
use rand_distr::{Binomial, ChiSquared, Distribution, Normal};

#[derive(Clone, Debug)]
pub enum RDistrib {
    MinMax(f32, f32),
    Normal(f32, f32),
    ChiSquare { k: f32, mult: f32, add: f32 },
    List { values: Vec<f32> },
    // WeightedList { values: Vec<f32> },
}

impl RDistrib {
    pub fn next(&self, rng: &mut StdRng) -> f32 {
        match self {
            RDistrib::MinMax(min, max) => rng.gen_range(*min..*max),

            RDistrib::Normal(mean, std_dev) => {
                let normal = Normal::new(*mean, *std_dev).unwrap();
                normal.sample(rng) as f32
            }

            RDistrib::ChiSquare { k, mult, add } => {
                let d = ChiSquared::new(*k).unwrap();
                (d.sample(rng) * *mult + *add) as f32
            }

            RDistrib::List { values } => {
                let index = rng.gen_range(0..values.len());
                values[index]
            }
        }
    }

    pub fn next_int(&self, rng: &mut StdRng) -> i32 {
        (self.next(rng).round() as i32).max(0)
    }
}
