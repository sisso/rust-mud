extern crate rand;
extern crate termion;
extern crate commons;
extern crate mud_domain;
extern crate socket_server;

pub mod game_server;

fn main() {
    crate::game_server::run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
    }
}
