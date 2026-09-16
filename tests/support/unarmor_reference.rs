//! Frozen pre-optimization implementation, shared by tests and benchmarks only.
//! Keep the function body unchanged to retain an independent baseline.
#[cfg(feature = "alloc")]
use crate::lib::std::{format, vec};
use crate::{errors::Result, lib, sentence::AisRawData};

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
    let mut offset = 0;
    for byte in data {
        let unarmored = match *byte {
            48..=87 => byte - 48,
            96..=119 => byte - 56,
            #[cfg(any(feature = "std", feature = "alloc"))]
            _ => return Err(format!("Value out of range: {}", byte).into()),
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            _ => return Err("Armored byte value out of range".into()),
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
        let final_idx = byte_count - 1;
        {
            let byte = &mut output[final_idx];
            let shift =
                (8 - bits_in_final_byte) + lib::std::cmp::min(fill_bits, bits_in_final_byte);
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
