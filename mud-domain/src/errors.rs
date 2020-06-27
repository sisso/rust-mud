use crate::game::loader::StaticId;
use serde::export::Formatter;
use std::any::Any;
use std::path::Display;

pub type Result<T> = std::result::Result<T, Error>;

/// Suffixes:
///
///  * Failure - This means that arguments or current state do not allow method to complete with success. It
/// is not considered a real error.
///  * Exception - Something is wrong, invalid argument or the current state. The algorithm expected something that
/// is true. It is recovery without side effects, but need to be logged and investigate.
///  * Error - Something bad happens like out of memory or disk error. Must cause system termination
#[derive(Debug)]
pub enum Error {
    NotFoundFailure,
    NotFoundStaticId(StaticId),
    InvalidStateFailure,
    InvalidArgumentFailure,
    Failure(String),
    NotFoundException,
    ConflictException,
    InvalidStateException,
    NotImplementedException,
    Exception(String),
    Error(String),
    IOError(std::io::Error),
}

impl Error {
    /// something is wrong
    pub fn is_exception(&self) -> bool {
        match self {
            Error::NotFoundException
            | Error::ConflictException
            | Error::Exception(_)
            | Error::NotImplementedException => true,
            _ => false,
        }
    }

    pub fn is_failure(&self) -> bool {
        !(self.is_fatal() || self.is_exception())
    }

    pub fn is_fatal(&self) -> bool {
        match self {
            Error::Error(_) | Error::IOError(_) => true,
            _ => false,
        }
    }

    fn as_failure(self) -> Self {
        if self.is_fatal() {
            panic!("fatal at {:?}", self);
        }

        if self.is_exception() {
            panic!("exception at {:?}", self);
        }

        self
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = format!("{:?}", self);
        write!(f, "{}", value);
        Ok(())
    }
}

pub trait ResultError<T> {
    fn as_failure(self) -> Result<T>;
}

impl<T> ResultError<T> for Result<T> {
    fn as_failure(self) -> Result<T> {
        self.map_err(|error| error.as_failure())
    }
}

pub trait AsResult<T> {
    fn as_result(self) -> Result<T>;
}

impl<T> AsResult<T> for Option<T> {
    fn as_result(self) -> Result<T> {
        self.ok_or(Error::NotFoundFailure)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}
