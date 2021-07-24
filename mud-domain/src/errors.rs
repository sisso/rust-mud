use crate::game::loader::dto::StaticId;
use std::any::Any;
use std::fmt::Formatter;
use std::path::Display;

// TODO: rename for a less conflicting
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
    // normal failures
    NotFoundFailure,
    NotFoundStaticId(StaticId),
    InvalidStateFailure,
    InvalidArgumentFailure,
    InvalidArgumentFailureStr(String),
    Failure(String),
    ConflictFailure,
    // warning exceptions
    NotFoundException,
    ConflictException,
    InvalidStateException,
    NotImplementedException,
    Exception(String),
    // errors are problems
    Error(String),
    IOError(std::io::Error),
    ParserError(serde_json::Error),
}

impl Error {
    /// something is wrong
    pub fn is_exception(&self) -> bool {
        match self {
            Error::NotFoundException
            | Error::ConflictException
            | Error::InvalidStateException
            | Error::NotImplementedException
            | Error::Exception(_) => true,
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
        write!(f, "{}", value)
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
    fn as_result_str(self, reason: &str) -> Result<T>;
    fn as_result_string<F: FnOnce() -> String>(self, reason: F) -> Result<T>;
    fn as_result_exception(self) -> Result<T>;
    fn as_exception_str(self, reason: String) -> Result<T>;
}

impl<T> AsResult<T> for Option<T> {
    fn as_result(self) -> Result<T> {
        self.ok_or(Error::NotFoundFailure)
    }

    fn as_result_str(self, reason: &str) -> Result<T> {
        self.ok_or(Error::Failure(reason.to_string()))
    }

    fn as_result_string<F: FnOnce() -> String>(self, reason: F) -> Result<T> {
        self.ok_or_else(|| Error::Failure(reason()))
    }

    fn as_exception_str(self, reason: String) -> Result<T> {
        self.ok_or(Error::Exception(reason))
    }

    fn as_result_exception(self) -> Result<T> {
        self.ok_or(Error::NotFoundException)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Exception(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Exception(s)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::ParserError(error)
    }
}
