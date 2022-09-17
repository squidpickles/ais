//! Common parsers
use crate::errors::Result;
use crate::lib;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
use super::nom_noalloc::count;
use lib::std::string::String;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::{map, map_res};
use nom::error::ErrorKind;
#[cfg(any(feature = "std", feature = "alloc"))]
use nom::multi::count;
use nom::IResult;

#[cfg(feature = "alloc")]
use crate::lib::std::{format, string::ToString};

/// This is the maximum number of bytes that a 6-bit ASCII representation can turn into.
/// The largest we see anywhere is 120 bits = 20 ASCII bytes.
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
const MAX_6BIT_ARRAY_BYTES: usize = 20;

pub fn parse_year(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u16>> {
    map(take_bits(14u16), |year| match year {
        0 => None,
        _ => Some(year),
    })(data)
}

pub fn parse_month(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits(4u8), |month| match month {
        0 => None,
        _ => Some(month),
    })(data)
}

pub fn parse_day(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits(5u8), |day| match day {
        0 => None,
        _ => Some(day),
    })(data)
}

pub fn parse_hour(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    take_bits(5u8)(data)
}

pub fn parse_minsec(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits(6u8), |minsec| match minsec {
        60 => None,
        _ => Some(minsec),
    })(data)
}

/// Returns the number of bits available to read, without otherwise modifying anything
pub fn remaining_bits(data: (&[u8], usize)) -> usize {
    data.0.len() * 8 - data.1
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type AsciiString = String<MAX_6BIT_ARRAY_BYTES>;
#[cfg(any(feature = "std", feature = "alloc"))]
pub type AsciiString = String;

/// Converts a number of bits, represented as 6-bit ASCII, into a String
pub fn parse_6bit_ascii(
    input: (&[u8], usize),
    size: usize,
) -> IResult<(&[u8], usize), AsciiString> {
    let char_count = size / 6;
    #[cfg(any(feature = "std", feature = "alloc"))]
    let (input, bytes) = count(map_res(take_bits(6u8), sixbit_to_ascii), char_count)(input)?;
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    let (input, bytes) = count::<_, _, _, _, MAX_6BIT_ARRAY_BYTES>(
        map_res(take_bits(6u8), sixbit_to_ascii),
        char_count,
    )(input)?;
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
        lib::std::str::from_utf8(&bytes)
            .map(|val| {
                (
                    input,
                    val.trim_start()
                        .trim_end_matches('@')
                        .trim_end()
                        .to_string(),
                )
            })
            .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::AlphaNumeric)))
    }
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    {
        lib::std::str::from_utf8(&bytes)
            .map(|val| {
                (
                    input,
                    val.trim_start().trim_end_matches('@').trim_end().into(),
                )
            })
            .map_err(|_| nom::Err::Failure(nom::error::Error::new(input, ErrorKind::AlphaNumeric)))
    }
}

/// Gets the message type from the first byte of supplied data
#[inline]
pub fn message_type(data: &[u8]) -> IResult<&[u8], u8> {
    bits(message_type_bits)(data)
}

/// Gets the message type from the current bitstream position
#[inline]
pub fn message_type_bits(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    take_bits(6u8)(data)
}

#[inline]
fn sixbit_to_ascii(data: u8) -> Result<u8> {
    #[cfg(any(feature = "std", feature = "alloc"))]
    match data {
        0..=31 => Ok(data + 64),
        32..=63 => Ok(data),
        _ => Err(format!("Illegal 6-bit character: {}", data).into()),
    }

    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    match data {
        0..=31 => Ok(data + 64),
        32..=63 => Ok(data),
        _ => Err("Illegal 6-bit character".into()),
    }
}

/// Converts a `0` to `false`, `1` to `true`. Expects only a single bit, so
/// other values will cause a panic.
#[inline]
pub fn u8_to_bool(data: u8) -> bool {
    match data {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}

pub fn signed_i32(input: (&[u8], usize), len: usize) -> IResult<(&[u8], usize), i32> {
    assert!(len <= lib::std::mem::size_of::<i32>() * 8);
    let (input, num) = take_bits::<_, i32, _, _>(len)(input)?;
    let mask = !0i32 << len;
    Ok((
        input,
        match (num << (32 - len)).leading_zeros() {
            0 => num | mask,
            _ => !mask & num,
        },
    ))
}
