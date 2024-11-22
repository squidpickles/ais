//! AIS parsing library for reading and decoding AIS NMEA sentences, including those with Tag Blocks.
//!
//! Given an NMEA stream, this library can extract and parse various AIS message types in detail.
//!
//! # Example:
//! ```
//! use ais::{AisFragments, AisParser};
//! use ais::messages::AisMessage;
//!
//! // The line below is an NMEA sentence, much like you'd see coming from an AIS decoder.
//! let line = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01";
//!
//! let mut parser = AisParser::new();
//! if let (None, AisFragments::Complete(sentence)) = parser.parse(line, true)? {
//!     // This sentence is complete, i.e., unfragmented
//!     assert_eq!(sentence.num_fragments, 1);
//!     // The data was transmitted on AIS channel B
//!     assert_eq!(sentence.channel, Some('B'));
//!
//!     if let Some(message) = sentence.message {
//!         match message {
//!             AisMessage::AidToNavigationReport(report) => {
//!                 assert_eq!(report.mmsi, 993692028);
//!                 assert_eq!(report.name, "SF OAK BAY BR VAIS E");
//!                 // There are many more fields available here
//!             },
//!             _ => panic!("Unexpected message type"),
//!         }
//!     }
//! }
//! # Ok::<(), ais::errors::Error>(())
//! ```
//!
//! This library also handles parsing AIS messages that include Tag Blocks, which provide additional metadata for the message.
//!
//!
//! # Main Utilities
//!
//! This library also includes utilities for decoding AIS messages from various sources, including UDP streams, TCP streams, files, or a single AIS message passed as an argument.
//!
//! # Usage
//!
//! Run the program with one of the following options:
//!
//! - **UDP Mode**: Listen for AIS messages over a UDP stream.
//!   ```sh
//!   cargo run -- --udp <ADDRESS>
//!   ```
//!   Replace `<ADDRESS>` with the IP and port to bind the UDP listener (e.g., `127.0.0.1:12345`).
//!
//! - **TCP Mode**: Connect to a TCP server to receive and decode AIS messages.
//!   ```sh
//!   cargo run -- --tcp <ADDRESS>
//!   ```
//!   Replace `<ADDRESS>` with the IP and port of the TCP server (e.g., `127.0.0.1:12346`).
//!
//! - **File Mode**: Decode AIS messages from a file.
//!   ```sh
//!   cargo run -- --file <PATH>
//!   ```
//!   Replace `<PATH>` with the path to the file containing AIS messages.
//!
//! - **Single Message Mode**: Decode a single AIS message directly from the command line.
//!   ```sh
//!   cargo run -- --message <AIS_MESSAGE>
//!   ```
//!   Replace `<AIS_MESSAGE>` with the actual AIS message string to decode.
//!
//! # Arguments
//!
//! - `--udp` / `-u <ADDRESS>`: The address (IP:port) to listen for UDP messages.
//! - `--tcp` / `-t <ADDRESS>`: The address (IP:port) to connect for TCP messages.
//! - `--file` / `-f <PATH>`: The file path to read AIS messages from.
//! - `--message` / `-m <AIS_MESSAGE>`: A single AIS message string to decode.
//!
//! # Example
//!
//! Decode a single AIS message:
//! ```sh
//! cargo run -- --message "!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05"
//! ```
//!
//! Decode AIS messages from a file:
//! ```sh
//! cargo run -- --file /path/to/ais_messages.txt
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
/// standard library stuff available crate-wide, regardless of `no_std` state
pub mod lib {
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    pub mod std {
        pub use core::{borrow, cmp, fmt, mem, result, str};

        pub mod vec {
            pub use heapless::Vec;
        }

        pub mod string {
            pub use heapless::String;
        }

        pub trait Error: fmt::Debug + fmt::Display {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }
        }
    }
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    pub mod std {
        extern crate alloc;
        pub use alloc::{borrow, fmt, format, str, string, vec};
        pub use core::{cmp, mem, result};

        pub trait Error: fmt::Debug + fmt::Display {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                None
            }
        }
    }

    #[cfg(feature = "std")]
    pub mod std {
        #[doc(hidden)]
        pub use std::{borrow, cmp, error, fmt, format, io, mem, result, str, string, vec};
    }
}

pub mod decoders;
pub mod errors;
pub mod messages;
pub mod sentence;
pub use decoders::utils::{decode, decode_from_file, decode_from_tcp, decode_from_udp};

pub use errors::Result;
pub use sentence::{AisFragments, AisParser};

#[cfg(test)]
mod test_helpers {
    #[inline]
    /// Compares two `f32`s, assuming they are both numeric, and panics if they differ
    pub fn f32_equal_naive(a: f32, b: f32) {
        if (a - b).abs() >= f32::EPSILON {
            panic!("float {} != {}", a, b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MESSAGES: [&[u8]; 8] = [
        b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01",
        b"!AIVDM,1,1,,A,403OtVAv6s5l1o?I``E`4I?02<34,0*21",
        b"!AIVDM,1,1,,B,ENkb9U79PW@80Q67h10dh1T6@Hq;`0W8:peOH00003vP000,0*1C",
        b"!AIVDM,1,1,,A,ENkb9H2`:@17W4b0h@@@@@@@@@@;WSEi:lK9800003vP000,0*08",
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A",
        b"!AIVDM,1,1,,B,403OtVAv6s5lOo?I`pE`4KO02<34,0*3E",
        b"!AIVDM,2,1,1,B,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP00000,0*78",
        b"!AIVDM,2,2,1,B,0000000,2*26",
    ];

    #[test]
    fn end_to_end() {
        let mut parser = sentence::AisParser::new();
        for line in TEST_MESSAGES.iter() {
            parser.parse(line, true).unwrap();
        }
    }
}
