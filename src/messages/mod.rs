use nom::{anychar, digit, IResult, hex_u32};
use errors::*;
use super::sentence::AisSentence;
use std::cmp;

pub type BitCount = usize;

pub struct AisMessage<'a> {
    bitstream: &'a [u8],
}

impl<'a> AisMessage<'a> {
    pub fn new_multi(sentence: &'a [&'a AisSentence]) -> Result<AisMessage<'a>> {
        unimplemented!()
    }

    pub fn new(sentence: &'a AisSentence) -> Result<AisMessage<'a>> {
        //AisMessage::new_multi(&[sentence])
        unimplemented!()
    }
    pub fn parse(data: &'a [u8], total_length: BitCount) -> Result<AisMessage<'a>> {
        unimplemented!()
    }

    fn unarmor_ais_data(data: &'a [u8], total_bit_length: BitCount) -> Vec<u8> {
        unimplemented!()
    }
}

fn unarmor<'a>(data: &'a [u8], fill_bits: BitCount) -> Result<Vec<u8>> {
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
}
