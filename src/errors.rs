//! Custom error types used by this crate

pub use err::*;

#[cfg(any(feature = "std", feature = "alloc"))]
mod err {
    use crate::lib;
    use lib::std::format;
    use lib::std::string::{String, ToString};

    /// Custom `Result` to prepopulate `Error` type
    pub type Result<T> = lib::std::result::Result<T, Error>;
    /// A general error in parsing an AIS message
    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Error {
        //#[error("invalid NMEA sentence: '{msg}'")]
        Nmea { msg: String },
        //#[error("checksum mismatch; expected: {expected:#X}, received: {found:#X}")]
        Checksum { expected: u8, found: u8 },
    }

    #[cfg(feature = "std")]
    impl std::error::Error for Error {}

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let msg = match self {
                Self::Nmea { msg } => format!("Error parsing NMEA content: {}", msg),
                Self::Checksum { expected, found } => {
                    format!("Checksum error; expected 0x{expected:x}, found 0x{found:x}")
                }
            };
            f.write_str(&msg)
        }
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

    impl<T: lib::std::fmt::Debug> From<nom::Err<nom::error::Error<T>>> for Error {
        fn from(err: nom::Err<nom::error::Error<T>>) -> Self {
            Self::Nmea {
                msg: format!("{:?}", err),
            }
        }
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
mod err {
    use crate::lib;

    /// Custom `Result` to prepopulate `Error` type
    pub type Result<T> = lib::std::result::Result<T, Error>;
    /// A general error in parsing an AIS message
    #[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub enum Error {
        //#[error("invalid NMEA sentence: '{msg}'")]
        Nmea { msg: &'static str },
        //#[error("checksum mismatch; expected: {expected:#X}, received: {found:#X}")]
        Checksum { expected: u8, found: u8 },
    }

    impl From<&'static str> for Error {
        fn from(err: &'static str) -> Self {
            Self::Nmea { msg: err }
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
}
