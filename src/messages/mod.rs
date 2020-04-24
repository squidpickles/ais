//! Specific AIS message types
use crate::errors::*;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::peek;
use nom::IResult;
use std::cmp;

pub mod aid_to_navigation_report;
pub mod base_station_report;
mod common;
mod navigation;
pub mod position_report;
mod radio_status;

/// Contains all structured messages recognized by this crate
#[derive(Debug)]
pub enum AisMessage {
    PositionReport(position_report::PositionReport),
    BaseStationReport(base_station_report::BaseStationReport),
    AidToNavigationReport(aid_to_navigation_report::AidToNavigationReport),
}

/// Trait that describes specific types of AIS messages
pub trait AisMessageType<'a>: Sized {
    /// The common name for the message type
    fn name(&self) -> &'static str;
    /// Converts a raw AIS message into a structured, queryable version
    fn parse(data: &'a [u8]) -> Result<Self>;
}

fn message_type(data: &[u8]) -> IResult<&[u8], u8> {
    bits(move |data| -> IResult<_, _> { peek(take_bits(6u8))(data) })(data)
}

/// Given an unarmored bitstream (see `unarmor()` for details), this
/// will return a message type object, if supported by this library
/// and the message is valid.
///
pub fn parse(unarmored: &[u8]) -> Result<AisMessage> {
    let (_, result) = message_type(unarmored)?;
    match result {
        1..=3 => Ok(AisMessage::PositionReport(
            position_report::PositionReport::parse(&unarmored)?,
        )),
        4 => Ok(AisMessage::BaseStationReport(
            base_station_report::BaseStationReport::parse(&unarmored)?,
        )),
        21 => Ok(AisMessage::AidToNavigationReport(
            aid_to_navigation_report::AidToNavigationReport::parse(&unarmored)?,
        )),
        _ => Err(format!("Unimplemented type: {}", result).into()),
    }
}

#[inline]
fn sixbit_to_ascii(data: u8) -> Result<u8> {
    match data {
        0..=31 => Ok(data + 64),
        32..=63 => Ok(data),
        _ => Err(format!("Illegal 6-bit character: {}", data).into()),
    }
}

#[inline]
fn u8_to_bool(data: u8) -> Result<bool> {
    match data {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("Invalid boolean value: {}", data).into()),
    }
}

fn signed_i32(input: (&[u8], usize), len: usize) -> IResult<(&[u8], usize), i32> {
    assert!(len <= ::std::mem::size_of::<i32>() * 8);
    let (input, num) = take_bits::<_, i32, _, (_, _)>(len)(input)?;
    let mask = !0i32 << len;
    Ok((
        input,
        match (num << (32 - len)).leading_zeros() {
            0 => num | mask,
            _ => !mask & num,
        },
    ))
}

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
/// See https://gpsd.gitlab.io/gpsd/AIVDM.html for more details.
pub fn unarmor(data: &[u8], fill_bits: usize) -> Result<Vec<u8>> {
    let bit_count = data.len() * 6;
    let byte_count = (bit_count / 8) + ((bit_count % 8 != 0) as usize);
    let mut output = vec![0; byte_count];
    let mut offset = 0;
    for byte in data {
        let unarmored = match *byte {
            48..=87 => byte - 48,
            96..=119 => byte - 56,
            _ => return Err(format!("Value out of range: {}", byte).into()),
        } << 2;
        let offset_byte = offset / 8;
        let offset_bit = offset % 8;
        output[offset_byte] |= unarmored >> offset_bit;
        if offset_bit > 2 {
            // Continue into the next byte
            output[offset_byte + 1] |= unarmored << (8 - offset_bit);
        }
        offset += 6;
    }
    if fill_bits != 0 {
        let bits_in_final_byte = match bit_count % 8 {
            0 => 8,
            1..=7 => bit_count % 8,
            _ => unreachable!(),
        };
        let final_idx = output.len() - 1;
        {
            let byte = &mut output[final_idx];
            let shift = (8 - bits_in_final_byte) + cmp::min(fill_bits, bits_in_final_byte);
            *byte &= match shift {
                0..=7 => 0xffu8 << shift,
                8 => 0x0u8,
                _ => unreachable!(),
            };
        }
        if fill_bits > bits_in_final_byte {
            let byte = &mut output[final_idx - 1];
            *byte &= 0xffu8 << (fill_bits - bits_in_final_byte);
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

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
