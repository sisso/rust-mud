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

pub mod server;
pub mod server_dummy;
pub mod server_socket;
pub mod game;
