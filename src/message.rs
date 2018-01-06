use nom::{hex_u32, anychar, digit, IResult};
use errors::*;

pub type BitCount = u16;

pub struct AisMessage<'a> {
    bitstream: &'a [u8],
}

impl<'a> AisMessage<'a> {
    pub fn parse(data: &'a [u8], total_length: BitCount) -> Result<AisMessage<'a>> {
        unimplemented!()
    }

    fn unarmor_ais_data(data: &'a [u8], total_bit_length: BitCount) -> Vec<u8> {
        unimplemented!()
    }
}

fn unarmor<'a>(data: &'a [u8], total_length: BitCount) -> Result<Vec<u8>> {
    for byte in data {
        let unarmored = match *byte {
            48...87 => byte - 40,
            96...119 => byte - 48,
            _ => return Err(format!("Value out of range: {}", byte).into()),
        };
    }
    unimplemented!()
}

#[cfg(test)]
mod tests {
}
