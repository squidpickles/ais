//! Binary Addressed Message (type 6)
use super::parsers::{remaining_bits, u8_to_bool};
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
pub struct BinaryAddressedMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub seqno: u8,
    pub dest_mmsi: u32,
    pub retransmit: bool,
    pub dac: u16,
    pub fid: u8,
    pub data: Vec<u8>,
}

impl<'a> AisMessageType<'a> for BinaryAddressedMessage {
    fn name(&self) -> &'static str {
        "Binary Addressed Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], BinaryAddressedMessage> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, seqno) = take_bits(2u8)(data)?;
        let (data, dest_mmsi) = take_bits(30u32)(data)?;
        let (data, retransmit) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(1u8)(data)?;
        let (data, dac) = take_bits(10u16)(data)?;
        let (data, fid) = take_bits(6u8)(data)?;

        // Handle the remaining bits for `data` as a Vec<u8>
        let remaining_bits = remaining_bits(data);
        let mut payload = Vec::with_capacity(remaining_bits / 8);
        let mut current_data = data;

        for _ in 1..(remaining_bits / 8) {
            let (next_data, byte) = take_bits(8u8)(current_data)?;
            payload.push(byte);
            current_data = next_data;
        }

        Ok((
            current_data,
            BinaryAddressedMessage {
                message_type,
                repeat_indicator,
                mmsi,
                seqno,
                dest_mmsi,
                retransmit,
                dac,
                fid,
                data: payload,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type6_example_1() {
        let bytestream = b"6B?n;be:cbapalgc;i6?Ow4";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let report = BinaryAddressedMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 6);
        assert_eq!(report.repeat_indicator, 1);
        assert_eq!(report.mmsi, 150834090);
        assert_eq!(report.seqno, 3);
        assert_eq!(report.dest_mmsi, 313240222);
        assert_eq!(report.retransmit, false);
        assert_eq!(report.dac, 669);
        assert_eq!(report.fid, 11);
        assert_eq!(report.data, vec![0xeb, 0x2f, 0x11, 0x8f, 0x7f, 0xf1]);
    }

    #[test]
    fn test_type6_example_2() {
        let bytestream = b"6>jR0600V:C0>da4P106P00";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let report = BinaryAddressedMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 6);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 992509976);
        assert_eq!(report.seqno, 0);
        assert_eq!(report.dest_mmsi, 2500912);
        assert_eq!(report.retransmit, false);
        assert_eq!(report.dac, 235);
        assert_eq!(report.fid, 10);
        assert_eq!(report.data, vec![0x44, 0x80, 0x10, 0x06, 0x80, 0x00]);
    }
}
