use std::any::Any;
use serde::export::Formatter;

pub type Result<T> = std::result::Result<T, Error>;

/// Project centralized error structure.
#[derive(Debug)]
pub enum Error {
    ObjIdNotFound(u32),
    StaticIdNotFound(u32),
    Conflict,
    IO(std::io::ErrorKind),
    ParserError { kind: String, value: String },
    Generic(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

impl From<String> for Error {
    fn from(string: String) -> Self {
        Error::Generic(string)
    }
}

