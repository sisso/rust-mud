use rand::{thread_rng, RngCore};

fn main() {
    let value = thread_rng().next_u64();
    println!("{}", value);
}