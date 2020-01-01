use serde::export::Formatter;
use std::any::Any;

pub type Result<T> = std::result::Result<T, Error>;

/// Suffixes:
///
///  * Failure - This means that arguments or current state do not allow method to complete with success. It
/// is not considered a real error.
///  * Exception - Something is wrong, invalid argument or the current state. The algorithm expected something that
/// is true. It is recovery without side effects, but need to be logged and investigate.
///  * Error - Something bad happens like out of memory or disk error. Must cause system termination
#[derive(Debug, Clone)]
pub enum Error {
    NotFoundFailure,
    InvalidStateFailure,
    InvalidArgumentFailure,
    Failure(String),
    NotFoundException,
    ConflictException,
    InvalidStateException,
    NotImplementedException,
    Exception(String),
    Error(String),
    IOError(std::io::ErrorKind),
}

impl Error {
    pub fn is_exception(&self) -> bool {
        match self {
            Error::NotFoundException | Error::ConflictException | Error::Exception(_) |
            Error::NotImplementedException => true,
            _ => false,
        }
    }

    pub fn is_fatal(&self) -> bool {
       match self {
           Error::Error(_) |  Error::IOError(_) => true,
           _ => false,
       }
    }
}

//impl From<&str> for Error {
//    fn from(s: &str) -> Self {
//        Error::Generic(s.to_string())
//    }
//}
//
//impl From<String> for Error {
//    fn from(string: String) -> Self {
//        Error::Generic(string)
//    }
//}

pub trait AsResult<T> {
    fn as_result(self) -> Result<T>;
}

impl<T> AsResult<T> for Option<T> {
    fn as_result(self) -> Result<T> {
        self.ok_or(Error::NotFoundFailure)
    }
}

//trait ResultExtra<T> {
//    fn when_err<O: FnOnce<Error>>(self, f: O) -> Result<T>;
//}
//
//impl<T, 'a> ResultExtra<&T> for Result<T> {
//    fn when_err<O: FnOnce<&'a Error>>(&'a self, f: O) -> Result<T> {
//        match self {
//            ok @ Ok(_)  => ok,
//            er@ Err(ref e) => {
//                f(e);
//                er
//            }
//        }
//    }
//}
