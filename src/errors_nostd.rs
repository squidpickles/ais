//! Custom error types used by this crate
use crate::lib;
use lib::std::format;
use lib::std::string::{String, ToString};

/// Custom `Result` to prepopulate `Error` type
pub type Result<T> = lib::std::result::Result<T, Error>;
/// A general error in parsing an AIS message
#[derive(Debug)]
pub enum Error {
    //#[error("invalid NMEA sentence: '{msg}'")]
    Nmea { msg: &'static str },
    //#[error("checksum mismatch; expected: {expected:#X}, received: {found:#X}")]
    Checksum { expected: u8, found: u8 },
}

impl From<&'static str> for Error {
    fn from(err: &str) -> Self {
        Self::Nmea { msg: err }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Nmea { msg: err }
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> From<nom::Err<T>> for Error {
    fn from(err: nom::Err<T>) -> Self {
        let err_str = match err {
            nom::Err::Incomplete(_) => "Incomplete data",
            nom::Err::Error(_) => "Parser error",
            nom::Err::Failure(_) => "Parser unrecoverable failure",
        };
        Self::Nmea { msg: err_str }
    }
}

impl<T> From<nom::Err<T>> for Error {
    fn from(err: nom::Err<T>) -> Self {
        let err_str = match err {
            nom::Err::Incomplete(_) => "Incomplete data",
            nom::Err::Error(_) => "Parser error",
            nom::Err::Failure(_) => "Parser unrecoverable failure",
        };
        Self::Nmea { msg: err_str }
    }
}

impl From<nom::Err<(&[u8], nom::error::ErrorKind)>> for Error {
    fn from(err: nom::Err<(&[u8], nom::error::ErrorKind)>) -> Self {
        Self::Nmea {
            msg: err.to_string(),
        }
    }
}

impl<T: lib::std::fmt::Debug> From<nom::Err<nom::error::Error<T>>> for Error {
    fn from(err: nom::Err<nom::error::Error<T>>) -> Self {
        Self::Nmea {
            msg: format!("{:?}", err),
        }
    }
}
