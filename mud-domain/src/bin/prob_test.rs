use itertools::Itertools;
use rand::thread_rng;
use rand_distr::Distribution;
use std::io::BufRead;

fn main() {
    let mut rng = thread_rng();
    let mut input = String::new();
    let mult: f32 = std::env::args().nth(1).unwrap().parse().unwrap();
    let add: f32 = std::env::args().nth(2).unwrap().parse().unwrap();

    loop {
        println!();
        let mut all = vec![];
        for i in 0..100 {
            let d = rand_distr::ChiSquared::new(3.0).unwrap();
            let v: f32 = d.sample(&mut rng) * mult + add;
            let v: i32 = v.round().max(0.0) as i32;
            print!("{} ", v);
            if i % 10 == 0 {
                println!()
            }

            all.push(v);
        }
        println!();
        all.sort();
        let grouped = all.iter().group_by(|i| **i);
        for g in &grouped {
            println!("{}: {}", g.0, g.1.count());
        }

        println!();

        std::io::stdin().lock().read_line(&mut input).unwrap();
        match input.as_str() {
            "q" => break,
            _ => {}
        }
    }
}
