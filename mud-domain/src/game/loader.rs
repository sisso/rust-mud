use std::path::Path;
use commons::save::Load;

pub mod scenery_space;
pub mod scenery_fantasy;
pub mod hocon_loader;

#[derive(Debug)]
pub enum LoaderError {
    Unknown
}
impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unknown error when parsing a configuration file")
    }
}

impl std::error::Error for LoaderError {
    fn description(&self) -> &str {
        "unknown error when parsing a configuration file"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Loader {
    fn load(path: &Path) -> Result<Box<dyn Load>>;
}
