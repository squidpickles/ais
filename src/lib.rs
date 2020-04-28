//! AIS parsing library, for reading AIS NMEA sentences
//!
//! Given an NMEA stream, this library can extract various AIS message types in more detail.
//!
//! # Example:
//! ```
//! use ais::{AisFragments, AisParser};
//! use ais::messages::AisMessage;
//!
//! // The line below is an NMEA sentence, much as you'd see coming out of an AIS decoder.
//! let line = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01";
//!
//! let mut parser = AisParser::new();
//! if let AisFragments::Complete(sentence) = parser.parse(line, true)? {
//!     // This sentence is complete, ie unfragmented
//!     assert_eq!(sentence.num_fragments, 1);
//!     // The data was transmitted on AIS channel B
//!     assert_eq!(sentence.channel, Some('B'));
//!
//!     if let Some(message) = sentence.message {
//!         match message {
//!             AisMessage::AidToNavigationReport(report) => {
//!                 assert_eq!(report.mmsi, 993692028);
//!                 assert_eq!(report.name, "SF OAK BAY BR VAIS E");
//!                 // There are a ton more fields available here
//!             },
//!             _ => panic!("Unexpected message type"),
//!         }
//!     }
//! }
//! # Ok::<(), ais::errors::Error>(())
//! ```
pub mod errors;
pub mod messages;
pub mod sentence;

pub use errors::Result;
pub use sentence::{AisFragments, AisParser};
#[cfg(test)]
mod test_helpers {
    #[inline]
    /// Compares two `f32`s, assuming they are both numeric, and panics if they differ
    pub fn f32_equal_naive(a: f32, b: f32) {
        if (a - b).abs() >= std::f32::EPSILON {
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
