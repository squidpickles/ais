//! Common parsers
use crate::errors::Result;

use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::{map, map_res};
use nom::error::ErrorKind;
use nom::multi::count;
use nom::IResult;

pub fn parse_year(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u16>> {
    map(take_bits::<_, _, _, (_, _)>(14u16), |year| match year {
        0 => None,
        _ => Some(year),
    })(data)
}

pub fn parse_month(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits::<_, _, _, (_, _)>(4u8), |month| match month {
        0 => None,
        _ => Some(month),
    })(data)
}

pub fn parse_day(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits::<_, _, _, (_, _)>(5u8), |day| match day {
        0 => None,
        _ => Some(day),
    })(data)
}

pub fn parse_hour(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    take_bits::<_, _, _, (_, _)>(5u8)(data)
}

pub fn parse_minsec(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map(take_bits::<_, _, _, (_, _)>(6u8), |minsec| match minsec {
        60 => None,
        _ => Some(minsec),
    })(data)
}

/// Returns the number of bits available to read, without otherwise modifying anything
pub fn remaining_bits(data: (&[u8], usize)) -> usize {
    data.0.len() * 8 - data.1
}

/// Converts a number of bits, represented as 6-bit ASCII, into a String
pub fn parse_6bit_ascii(input: (&[u8], usize), size: usize) -> IResult<(&[u8], usize), String> {
    let char_count = size / 6;
    let (input, bytes) = count(
        map_res(take_bits::<_, _, _, (_, _)>(6u8), sixbit_to_ascii),
        char_count,
    )(input)?;
    match ::std::str::from_utf8(&bytes) {
        Ok(val) => Ok((
            input,
            val.trim_start()
                .trim_end_matches('@')
                .trim_end()
                .to_string(),
        )),
        Err(_) => Err(nom::Err::Error((input, ErrorKind::AlphaNumeric))),
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
    match data {
        0..=31 => Ok(data + 64),
        32..=63 => Ok(data),
        _ => Err(format!("Illegal 6-bit character: {}", data).into()),
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
