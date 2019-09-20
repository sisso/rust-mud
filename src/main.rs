extern crate rand;
extern crate termion;

// need to be before log macros
pub mod utils;

#[macro_export]
macro_rules! debug {
    ($msg:expr) => (crate::utils::logs::debug(file!(), $msg));
    ($fmt:expr, $($arg:tt)*) => (crate::utils::logs::debug(file!(), format!($fmt, $($arg)*).as_ref()));
}

#[macro_export]
macro_rules! info {
    ($msg:expr) => (crate::utils::logs::info(file!(), $msg));
    ($fmt:expr, $($arg:tt)*) => (crate::utils::logs::info(file!(), format!($fmt, $($arg)*).as_ref()));
}

#[macro_export]
macro_rules! warn {
    ($msg:expr) => (crate::utils::logs::warn(file!(), $msg));
    ($fmt:expr, $($arg:tt)*) => (crate::utils::logs::warn(file!(), format!($fmt, $($arg)*).as_ref()));
}

pub mod game;
pub mod server;
pub mod game_server;

fn main() {
    crate::game_server::run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        debug!("one");
        debug!("{} {:?}", 1, "none");
        info!("one");
        info!("{} {:?}", 1, "none");
        assert_eq!(true, true);
    }
}
