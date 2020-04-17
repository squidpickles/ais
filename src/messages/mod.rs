use errors::*;
use nom::IResult;
use std::cmp;

pub mod aid_to_navigation_report;
pub mod base_station_report;
mod common;
mod navigation;
pub mod position_report;
mod radio_status;

pub type BitCount = usize;
pub type ByteStream<'a> = &'a [u8];
pub type BitStream<'a> = &'a [u8];

#[derive(Debug)]
pub enum AisMessage {
    PositionReport(position_report::PositionReport),
    BaseStationReport(base_station_report::BaseStationReport),
    AidToNavigationReport(aid_to_navigation_report::AidToNavigationReport),
}

pub trait AisMessageType<'a>: Sized {
    fn name(&self) -> &'static str;
    fn parse(data: &'a [u8]) -> Result<Self>;
}

named!(message_type<(&[u8], usize), u8>, peek!(take_bits!(u8, 6)));

pub fn parse(unarmored: BitStream) -> Result<AisMessage> {
    match message_type((unarmored, 0)) {
        IResult::Done(_, result) => Ok(match result {
            1...3 => {
                AisMessage::PositionReport(position_report::PositionReport::parse(&unarmored)?)
            }
            4 => AisMessage::BaseStationReport(base_station_report::BaseStationReport::parse(
                &unarmored,
            )?),
            21 => AisMessage::AidToNavigationReport(
                aid_to_navigation_report::AidToNavigationReport::parse(&unarmored)?,
            ),
            _ => Err(format!("Unimplemented type: {}", result))?,
        }),
        IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
        IResult::Incomplete(_) => Err("incomplete AIS sentence")?,
    }
}

fn u32_to_u8_array(data: u32) -> [u8; 4] {
    let b1: u8 = ((data >> 24) & 0xff) as u8;
    let b2: u8 = ((data >> 16) & 0xff) as u8;
    let b3: u8 = ((data >> 8) & 0xff) as u8;
    let b4: u8 = (data & 0xff) as u8;
    return [b1, b2, b3, b4];
}

#[inline]
fn sixbit_to_ascii(data: u8) -> Result<u8> {
    match data {
        0...31 => Ok(data + 64),
        32...63 => Ok(data),
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
    let parsed = try_parse!(input, take_bits!(i32, len));
    let input = parsed.1;
    let mask = !0i32 << len;
    IResult::Done(
        parsed.0,
        match (input << (32 - len)).leading_zeros() {
            0 => input | mask,
            _ => !mask & input,
        },
    )
}

pub fn unarmor<'a>(data: ByteStream, fill_bits: BitCount) -> Result<Vec<u8>> {
    let bit_count = data.len() * 6;
    let byte_count = (bit_count / 8) + ((bit_count % 8 != 0) as usize);
    let mut output = vec![0; byte_count];
    let mut offset = 0;
    for byte in data {
        let unarmored = match *byte {
            48...87 => byte - 48,
            96...119 => byte - 56,
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
            1...7 => bit_count % 8,
            _ => unreachable!(),
        };
        let final_idx = output.len() - 1;
        {
            let byte = &mut output[final_idx];
            let shift = (8 - bits_in_final_byte) + cmp::min(fill_bits, bits_in_final_byte);
            *byte &= match shift {
                0...7 => 0xffu8 << shift,
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
        assert_eq!([0b00100100,], &result[..]);
    }

    #[test]
    fn unarmor_single_byte_fill() {
        let input = b"9";
        let result = unarmor(input, 4).unwrap();
        assert_eq!([0b00000000,], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_unaligned() {
        let input = b"9q";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b00100111, 0b10010000,], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_aligned() {
        let input = b"9qKr";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b00100111, 0b10010110, 0b011111010], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_aligned_fill() {
        let input = b"9qWr";
        let result = unarmor(input, 4).unwrap();
        assert_eq!([0b00100111, 0b10011001, 0b11110000], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_unaligned_fill() {
        let input = b"9qW";
        let result = unarmor(input, 3).unwrap();
        assert_eq!([0b00100111, 0b10011000, 0b00000000], &result[..]);
    }
    // TODO: test parse i32
}
