//! Custom error types used by this crate
use thiserror::Error;

/// Custom `Result` to prepopulate `Error` type
pub type Result<T> = std::result::Result<T, Error>;

/// A general error in parsing an AIS message
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid NMEA sentence: '{msg}'")]
    Nmea { msg: String },
    #[error("checksum mismatch; expected: {expected:#X}, received: {found:#X}")]
    Checksum { expected: u8, found: u8 },
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Nmea { msg: err.into() }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Nmea { msg: err }
    }
}

impl From<nom::Err<&[u8]>> for Error {
    fn from(err: nom::Err<&[u8]>) -> Self {
        Self::Nmea {
            msg: err.to_string(),
        }
    }
}

impl From<nom::Err<(&[u8], nom::error::ErrorKind)>> for Error {
    fn from(err: nom::Err<(&[u8], nom::error::ErrorKind)>) -> Self {
        Self::Nmea {
            msg: err.to_string(),
        }
    }
}

impl<T: std::fmt::Debug> From<nom::Err<nom::error::Error<T>>> for Error {
    fn from(err: nom::Err<nom::error::Error<T>>) -> Self {
        Self::Nmea {
            msg: format!("{:?}", err),
        }
    }
}
