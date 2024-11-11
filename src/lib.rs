//! AIS parsing library for reading AIS NMEA sentences, with support for JSON serialization.
//!
//! This library parses NMEA AIS (Automatic Identification System) sentences and provides
//! structured representations of the data, allowing further processing or analysis.
//!
//! # Features
//! - Parses AIS NMEA sentences into structured types.
//! - Supports JSON serialization and deserialization for `AisSentence` objects.
//!
//! # Example
//! ```
//! use ais::{AisFragments, AisParser, serialize_to_json, deserialize_from_json};
//! use ais::messages::AisMessage;
//!
//! let line = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01";
//! let mut parser = AisParser::new();
//!
//! if let AisFragments::Complete(sentence) = parser.parse(line, true).unwrap() {
//!     assert_eq!(sentence.num_fragments, 1);
//!     assert_eq!(sentence.channel, Some('B'));
//!
//!     if let Some(ref message) = sentence.message {
//!         match message {
//!             AisMessage::AidToNavigationReport(report) => {
//!                 assert_eq!(report.mmsi, 993692028);
//!                 assert_eq!(report.name, "SF OAK BAY BR VAIS E");
//!             },
//!             _ => panic!("Unexpected message type"),
//!         }
//!     }
//!
//!     let json = serialize_to_json(&sentence).unwrap();
//!     let deserialized_sentence = deserialize_from_json(&json).unwrap();
//!     assert_eq!(sentence, deserialized_sentence);
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[doc(hidden)]
/// Standard library items, available crate-wide regardless of `no_std` state.
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

pub mod errors;
pub mod messages;
pub mod sentence;

pub use errors::Result;
pub use sentence::{AisFragments, AisParser};

use sentence::AisSentence;
use serde_json::Error as SerdeError;

/// Serializes an `AisSentence` to JSON
#[cfg(any(feature = "std", feature = "alloc"))]
use lib::std::string::String;
pub fn serialize_to_json(sentence: &AisSentence) -> std::result::Result<String, SerdeError> {
    serde_json::to_string(sentence)
}
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn deserialize_from_json(json_data: &str) -> std::result::Result<AisSentence, SerdeError> {
    serde_json::from_str(json_data)
}

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
    use crate::sentence::{AisReportType, AisSentence, TalkerId};

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
        let mut parser = AisParser::new();
        for line in TEST_MESSAGES.iter() {
            parser.parse(line, true).unwrap();
        }
    }

    #[test]
    fn test_json_serialization() {
        let mut parser = AisParser::new();
        let line = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01";

        if let AisFragments::Complete(sentence) = parser.parse(line, true).unwrap() {
            // Serialize the sentence to JSON
            let json = serialize_to_json(&sentence).expect("Failed to serialize to JSON");
            println!("Serialized JSON: {}", json);

            // Deserialize back from JSON
            let deserialized_sentence =
                deserialize_from_json(&json).expect("Failed to deserialize from JSON");

            assert_eq!(sentence, deserialized_sentence);
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        // Create a sample AisSentence struct
        let original_sentence = AisSentence {
            message: None,
            talker_id: TalkerId::AI,
            report_type: AisReportType::VDM,
            num_fragments: 1,
            fragment_number: 1,
            message_id: Some(123),
            channel: Some('A'),
            data: vec![69, 62, 107, 98, 57, 79], // sample data; replace with real data if needed
            fill_bit_count: 0,
            message_type: 1,
        };

        // Serialize to JSON
        let json_data = serialize_to_json(&original_sentence).expect("Serialization failed");

        // Deserialize back to an AisSentence
        let deserialized_sentence: AisSentence =
            deserialize_from_json(&json_data).expect("Deserialization failed");

        // Check if the deserialized struct matches the original
        assert_eq!(original_sentence, deserialized_sentence);
    }
}
