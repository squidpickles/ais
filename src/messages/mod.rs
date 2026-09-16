//! Specific AIS message types
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::errors::Error;
use crate::errors::Result;
use crate::lib;
use crate::sentence::AisRawData;

pub mod addressed_safety_related;
pub mod aid_to_navigation_report;
pub mod assignment_mode_command;
pub mod base_station_report;
pub mod binary_acknowledge;
pub mod binary_addressed;
pub mod binary_broadcast_message;
pub mod data_link_management_message;
pub mod dgnss_broadcast_binary_message;
pub mod extended_class_b_position_report;
pub mod interrogation;
pub mod long_range_ais_broadcast;
pub mod navigation;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
mod nom_noalloc;
mod parsers;
pub mod position_report;
pub mod radio_status;
pub mod safety_related_acknowledgment;
pub mod safety_related_broadcast;
pub mod standard_aircraft_position_report;
pub mod standard_class_b_position_report;
pub mod static_and_voyage_related_data;
pub mod static_data_report;
pub mod types;
pub mod utc_date_inquiry;
pub mod utc_date_response;

pub use parsers::message_type;

#[cfg(feature = "alloc")]
use crate::lib::std::{format, vec, vec::Vec};

/// Contains all structured messages recognized by this crate
#[derive(Debug, PartialEq)]
pub enum AisMessage {
    PositionReport(position_report::PositionReport),
    BaseStationReport(base_station_report::BaseStationReport),
    BinaryBroadcastMessage(binary_broadcast_message::BinaryBroadcastMessage),
    Interrogation(interrogation::Interrogation),
    StaticAndVoyageRelatedData(static_and_voyage_related_data::StaticAndVoyageRelatedData),
    DgnssBroadcastBinaryMessage(dgnss_broadcast_binary_message::DgnssBroadcastBinaryMessage),
    StandardClassBPositionReport(standard_class_b_position_report::StandardClassBPositionReport),
    ExtendedClassBPositionReport(extended_class_b_position_report::ExtendedClassBPositionReport),
    DataLinkManagementMessage(data_link_management_message::DataLinkManagementMessage),
    AidToNavigationReport(aid_to_navigation_report::AidToNavigationReport),
    StaticDataReport(static_data_report::StaticDataReport),
    UtcDateResponse(utc_date_response::UtcDateResponse),
    StandardAircraftPositionReport(standard_aircraft_position_report::SARPositionReport),
    AssignmentModeCommand(assignment_mode_command::AssignmentModeCommand),
    BinaryAcknowledgeMessage(binary_acknowledge::BinaryAcknowledge),
    UtcDateInquiry(utc_date_inquiry::UtcDateInquiry),
    AddressedSafetyRelatedMessage(addressed_safety_related::AddressedSafetyRelatedMessage),
    SafetyRelatedBroadcastMessage(safety_related_broadcast::SafetyRelatedBroadcastMessage),
    SafetyRelatedAcknowledgment(safety_related_acknowledgment::SafetyRelatedAcknowledge),
    LongRangeAisBroadcastMessage(long_range_ais_broadcast::LongRangeAisBroadcastMessage),
    BinaryAddressedMessage(binary_addressed::BinaryAddressedMessage),
}

/// Trait that describes specific types of AIS messages
pub trait AisMessageType<'a>: Sized {
    /// The common name for the message type
    fn name(&self) -> &'static str;
    /// Converts a raw AIS message into a structured, queryable version
    fn parse(data: &'a [u8]) -> Result<Self>;
}

/// Given an unarmored bitstream (see [`unarmor()`](fn.unarmor.html) for details), this
/// will return a message type object, if supported by this library
/// and the message is valid.
///
pub fn parse(unarmored: &[u8]) -> Result<AisMessage> {
    let (_, result) = message_type(unarmored)?;
    match result {
        1..=3 => Ok(AisMessage::PositionReport(
            position_report::PositionReport::parse(unarmored)?,
        )),
        4 => Ok(AisMessage::BaseStationReport(
            base_station_report::BaseStationReport::parse(unarmored)?,
        )),
        5 => Ok(AisMessage::StaticAndVoyageRelatedData(
            static_and_voyage_related_data::StaticAndVoyageRelatedData::parse(unarmored)?,
        )),
        7 => Ok(AisMessage::BinaryAcknowledgeMessage(
            binary_acknowledge::BinaryAcknowledge::parse(unarmored)?,
        )),
        6 => Ok(AisMessage::BinaryAddressedMessage(
            binary_addressed::BinaryAddressedMessage::parse(unarmored)?,
        )),
        8 => Ok(AisMessage::BinaryBroadcastMessage(
            binary_broadcast_message::BinaryBroadcastMessage::parse(unarmored)?,
        )),
        9 => Ok(AisMessage::StandardAircraftPositionReport(
            standard_aircraft_position_report::SARPositionReport::parse(unarmored)?,
        )),
        10 => Ok(AisMessage::UtcDateInquiry(
            utc_date_inquiry::UtcDateInquiry::parse(unarmored)?,
        )),
        11 => Ok(AisMessage::UtcDateResponse(
            utc_date_response::UtcDateResponse::parse(unarmored)?,
        )),
        12 => Ok(AisMessage::AddressedSafetyRelatedMessage(
            addressed_safety_related::AddressedSafetyRelatedMessage::parse(unarmored)?,
        )),
        13 => Ok(AisMessage::SafetyRelatedAcknowledgment(
            safety_related_acknowledgment::SafetyRelatedAcknowledge::parse(unarmored)?,
        )),
        14 => Ok(AisMessage::SafetyRelatedBroadcastMessage(
            safety_related_broadcast::SafetyRelatedBroadcastMessage::parse(unarmored)?,
        )),
        15 => Ok(AisMessage::Interrogation(
            interrogation::Interrogation::parse(unarmored)?,
        )),
        16 => Ok(AisMessage::AssignmentModeCommand(
            assignment_mode_command::AssignmentModeCommand::parse(unarmored)?,
        )),
        17 => Ok(AisMessage::DgnssBroadcastBinaryMessage(
            dgnss_broadcast_binary_message::DgnssBroadcastBinaryMessage::parse(unarmored)?,
        )),
        18 => Ok(AisMessage::StandardClassBPositionReport(
            standard_class_b_position_report::StandardClassBPositionReport::parse(unarmored)?,
        )),
        19 => Ok(AisMessage::ExtendedClassBPositionReport(
            extended_class_b_position_report::ExtendedClassBPositionReport::parse(unarmored)?,
        )),
        20 => Ok(AisMessage::DataLinkManagementMessage(
            data_link_management_message::DataLinkManagementMessage::parse(unarmored)?,
        )),
        21 => Ok(AisMessage::AidToNavigationReport(
            aid_to_navigation_report::AidToNavigationReport::parse(unarmored)?,
        )),
        24 => Ok(AisMessage::StaticDataReport(
            static_data_report::StaticDataReport::parse(unarmored)?,
        )),
        27 => Ok(AisMessage::LongRangeAisBroadcastMessage(
            long_range_ais_broadcast::LongRangeAisBroadcastMessage::parse(unarmored)?,
        )),
        #[cfg(any(feature = "std", feature = "alloc"))]
        _ => Err(Error::UnsupportedMessageType {
            message_type: result,
        }),
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        _ => Err("Unimplemented message type".into()),
    }
}

// Invalid characters use a sentinel outside the six-bit range.
const DEARMOR: [u8; 256] = {
    let mut table = [0xff; 256];
    let mut i = 48;
    while i <= 87 {
        table[i] = (i - 48) as u8;
        i += 1;
    }
    i = 96;
    while i <= 119 {
        table[i] = (i - 56) as u8;
        i += 1;
    }
    table
};

/// Converts 8-bit ASCII (armored) into packed 6-bit (unarmored) sequences.
///
/// AIS data is bit-, not byte-oriented. AIS data is split into 6-bit chunks,
/// which are then represented in ASCII as 8-bit characters. That process
/// is called "armoring"
///
/// The `fill_bits` parameter is a count of bits needed to pad
/// the complete message out to a 6-bit boundary. It should be supplied
/// as part of the main sentence.
///
/// Returns an error if any of the individual bytes cannot be converted
/// to a valid 6-bit chunk.
///
/// Uses lookup-based four-to-three packing. Algorithm developed by Henrik Holm.
/// Described in Holm and Mellegård,
/// *Fast Decoding of Automatic Identification Systems (AIS) Data*, COMPIT 2018:
/// <https://www.researchgate.net/publication/325301032>.
/// Preserves the forward byte order expected by this crate's parsers.
/// Supply a fill count of 0..=5, and zero for empty input.
/// See <https://gpsd.gitlab.io/gpsd/AIVDM.html> for more details.
pub fn unarmor(data: &[u8], fill_bits: usize) -> Result<AisRawData> {
    let bit_count = data.len() * 6;
    let byte_count = (bit_count / 8) + (!bit_count.is_multiple_of(8) as usize);
    #[cfg(any(feature = "std", feature = "alloc"))]
    let mut output = vec![0; byte_count];
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    let mut output = {
        let mut output = AisRawData::default();
        output
            .resize(byte_count, 0)
            .map_err(|_| crate::errors::Error::from("Unarmor output vector too large"))?;
        output
    };

    let mut chunks = data.chunks_exact(4);
    for (chunk, dst) in chunks.by_ref().zip(output.chunks_exact_mut(3)) {
        let a = DEARMOR[chunk[0] as usize];
        let b = DEARMOR[chunk[1] as usize];
        let c = DEARMOR[chunk[2] as usize];
        let d = DEARMOR[chunk[3] as usize];
        if (a | b | c | d) & 0xc0 != 0 {
            // Report the first invalid character, matching the original behavior.
            let byte = chunk
                .iter()
                .copied()
                .find(|&byte| DEARMOR[byte as usize] == 0xff)
                .unwrap();
            return Err(invalid_armored_byte(byte));
        }
        let packed = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | d as u32;
        dst[0] = (packed >> 16) as u8;
        dst[1] = (packed >> 8) as u8;
        dst[2] = packed as u8;
    }
    // At most three remaining characters; their bits are left aligned.
    let remainder = chunks.remainder();
    let mut packed = 0u32;
    for (i, &byte) in remainder.iter().enumerate() {
        let value = DEARMOR[byte as usize];
        if value == 0xff {
            return Err(invalid_armored_byte(byte));
        }
        packed |= (value as u32) << (18 - 6 * i);
    }
    let tail = &mut output[data.len() / 4 * 3..];
    let bytes = packed.to_be_bytes();
    tail.copy_from_slice(&bytes[1..1 + tail.len()]);

    // Preserve the original output length and fill-bit masking exactly.
    if fill_bits != 0 {
        let bits_in_final_byte = match bit_count % 8 {
            0 => 8,
            n => n,
        };
        let final_idx = byte_count - 1;
        let shift = (8 - bits_in_final_byte) + lib::std::cmp::min(fill_bits, bits_in_final_byte);
        output[final_idx] &= if shift == 8 { 0 } else { 0xffu8 << shift };
        if fill_bits > bits_in_final_byte {
            output[final_idx - 1] &= 0xffu8 << (fill_bits - bits_in_final_byte);
        }
    }
    Ok(output)
}

#[cold]
fn invalid_armored_byte(byte: u8) -> crate::errors::Error {
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
        format!("Value out of range: {}", byte).into()
    }
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    {
        let _ = byte;
        "Armored byte value out of range".into()
    }
}

#[cfg(test)]
#[path = "../../tests/support/unarmor_reference.rs"]
mod unarmor_reference;

#[cfg(any(feature = "std", feature = "alloc"))]
#[inline]
fn push_unwrap<T>(list: &mut Vec<T>, item: T) {
    list.push(item);
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[inline]
fn push_unwrap<T: core::fmt::Debug, const C: usize>(list: &mut lib::std::vec::Vec<T, C>, item: T) {
    list.push(item).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimized_matches_original_lengths_and_fill() {
        let mut data = [0u8; 512];
        let mut state = 123456789u32;
        for byte in &mut data {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let value = (state & 63) as u8;
            *byte = if value < 40 { value + 48 } else { value + 56 };
        }
        assert_eq!(
            unarmor(b"", 0).unwrap(),
            unarmor_reference::unarmor(b"", 0).unwrap()
        );
        for len in 1..=data.len() {
            for fill in 0..=5 {
                assert_eq!(
                    unarmor(&data[..len], fill).unwrap(),
                    unarmor_reference::unarmor(&data[..len], fill).unwrap(),
                    "length={len}, fill={fill}"
                );
            }
        }
    }

    #[test]
    fn optimized_all_byte_values_and_positions() {
        for len in 1..=12 {
            for pos in 0..len {
                for byte in 0..=255u8 {
                    let mut data = [b'0'; 12];
                    data[pos] = byte;
                    let expected = unarmor_reference::unarmor(&data[..len], 0);
                    let actual = unarmor(&data[..len], 0);
                    match (expected, actual) {
                        (Ok(a), Ok(b)) => assert_eq!(a, b),
                        (Err(a), Err(b)) => assert_eq!(a, b),
                        _ => panic!("mismatch: length={len}, position={pos}, byte={byte}"),
                    }
                }
            }
        }
    }

    #[test]
    fn unarmor_single_byte() {
        let input = b"9";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b0010_0100,], &result[..]);
    }

    #[test]
    fn unarmor_single_byte_fill() {
        let input = b"9";
        let result = unarmor(input, 4).unwrap();
        assert_eq!([0b0000_0000,], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_unaligned() {
        let input = b"9q";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b0010_0111, 0b1001_0000,], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_aligned() {
        let input = b"9qKr";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b0010_0111, 0b1001_0110, 0b0_1111_1010], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_long() {
        let input = b"E>kb9O9aS@7PUh";
        let result = unarmor(input, 4).unwrap();
        assert_eq!(
            [
                0b0101_0100,
                0b1110_1100,
                0b1110_1010,
                0b0010_0101,
                0b1111_0010,
                0b0110_1001,
                0b1000_1101,
                0b0000_0001,
                0b1110_0000,
                0b1001_0111,
                0b0000_0000,
            ],
            &result[..]
        );
    }

    #[test]
    fn unarmor_multi_bytes_aligned_fill() {
        let input = b"9qWr";
        let result = unarmor(input, 4).unwrap();
        assert_eq!([0b0010_0111, 0b1001_1001, 0b1111_0000], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_unaligned_fill() {
        let input = b"9qW";
        let result = unarmor(input, 3).unwrap();
        assert_eq!([0b0010_0111, 0b1001_1000, 0b0000_0000], &result[..]);
    }
    // TODO: test parse i32
}
