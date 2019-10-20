extern crate mud_domain;

use mud_domain::game::comm;

fn main() {
    println!();
    println!("{}", comm::help());
    println!();
}
