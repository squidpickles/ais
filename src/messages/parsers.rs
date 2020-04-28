//! Common parsers
use super::sixbit_to_ascii;

use nom::bits::complete::take as take_bits;
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::multi::count;
use nom::IResult;

pub fn parse_year(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u16>> {
    map_res(take_bits::<_, _, _, (_, _)>(14u16), |year| match year {
        0 => Ok(None),
        1..=9999 => Ok(Some(year)),
        _ => Err("Invalid year"),
    })(data)
}

pub fn parse_month(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map_res(take_bits::<_, _, _, (_, _)>(4u8), |month| match month {
        0 => Ok(None),
        1..=12 => Ok(Some(month)),
        _ => Err("Invalid month"),
    })(data)
}

pub fn parse_day(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map_res(take_bits::<_, _, _, (_, _)>(5u8), |day| match day {
        0 => Ok(None),
        1..=31 => Ok(Some(day)),
        _ => Err("Invalid day"),
    })(data)
}

pub fn parse_hour(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map_res(take_bits::<_, _, _, (_, _)>(5u8), |hour| match hour {
        0..=23 => Ok(Some(hour)),
        24 => Ok(None),
        _ => Err("Invalid hour"),
    })(data)
}

pub fn parse_minsec(data: (&[u8], usize)) -> IResult<(&[u8], usize), Option<u8>> {
    map_res(take_bits::<_, _, _, (_, _)>(6u8), |minsec| match minsec {
        0..=59 => Ok(Some(minsec)),
        60 => Ok(None),
        _ => Err("Invalid minute/second"),
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
