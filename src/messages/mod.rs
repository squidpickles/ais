use nom::{anychar, digit, IResult, hex_u32};
use errors::*;
use super::sentence::AisSentence;

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
    let byte_count = (data.len() * 6) / 8 + ((data.len() * 6) % 8 != 0) as usize;
    println!("Byte count is {}", byte_count);
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
            output[offset_byte + 1] |= unarmored << (8 - offset_bit);
        }
        offset += 6;
    }
    if fill_bits != 0 {
        if let Some(final_byte) = output.last_mut() {
            *final_byte &= 0b11111111u8 << fill_bits;
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
    fn unarmor_multi_bytes_aligned() {
        let input = b"9q";
        let result = unarmor(input, 0).unwrap();
        assert_eq!([0b00100111, 0b10010000,], &result[..]);
    }

    #[test]
    fn unarmor_multi_bytes_fill_bits() {
        let input = b"9qKr";
        let result = unarmor(input, 4).unwrap();
        assert_eq!([0b00100111, 0b10010110, 0b011110000], &result[..]);
    }
}
