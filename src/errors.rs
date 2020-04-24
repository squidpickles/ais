//! Custom error types used by this crate

use error_chain::*;
error_chain! {
    errors {
        /// Incorrect NMEA sentence format
        Nmea(msg: String) {
            description("invalid NMEA sentence")
            display("invalid NMEA sentence: '{}'", msg)
        }
        /// A checksum does not match its expected value
        Checksum(expected: u8, received: u8) {
            description("checksum mismatch")
            display("checksum mismatch; expected: {:#X}, received: {:#X}", expected, received)
        }
    }
}

impl From<nom::Err<&[u8]>> for Error {
    fn from(err: nom::Err<&[u8]>) -> Self {
        err.to_string().into()
    }
}

impl From<nom::Err<(&[u8], nom::error::ErrorKind)>> for Error {
    fn from(err: nom::Err<(&[u8], nom::error::ErrorKind)>) -> Self {
        err.to_string().into()
    }
}
