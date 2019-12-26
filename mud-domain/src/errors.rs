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
    IllegalState,
    IllegalArgument,
    IllegalArgumentMsg { msg: String },
    InCombat,
    IsResting,
    NotFound,
    CanNotBeEquipped,
		NotImplemented,
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

pub trait AsResult<T> {
    fn as_result(self) -> Result<T>;
}

impl<T> AsResult<T> for Option<T> {
    fn as_result(self) -> Result<T> {
        self.ok_or(Error::NotFound)
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

