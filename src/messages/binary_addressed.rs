//! Binary Addressed Message (type 6)
use super::parsers::u8_to_bool;
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
const MAX_DATA_SIZE_BYTES: usize = 119;

#[cfg(any(feature = "std", feature = "alloc"))]
pub type MessageData = lib::std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type MessageData = lib::std::vec::Vec<u8, MAX_DATA_SIZE_BYTES>;

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
    pub data: MessageData,
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
fn parse_base<'a>(data: &'a [u8]) -> IResult<&'a [u8], BinaryAddressedMessage> {
    bits(move |data: (&'a [u8], usize)| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, seqno) = take_bits(2u8)(data)?;
        let (data, dest_mmsi) = take_bits(30u32)(data)?;
        let (data, retransmit) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(1u8)(data)?;
        let (data, dac) = take_bits(10u16)(data)?;
        let (data, fid) = take_bits(6u8)(data)?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        let data_owned = data.0.into();
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let data_owned = data.0.try_into().map_err(|_| {
            nom::Err::Failure(nom::error::Error::new(
                data,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;
        Ok((
            (<&[u8]>::default(), 0),
            BinaryAddressedMessage {
                message_type,
                repeat_indicator,
                mmsi,
                seqno,
                dest_mmsi,
                retransmit,
                dac,
                fid,
                data: data_owned,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
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
        assert!(!report.retransmit);
        assert_eq!(report.dac, 669);
        assert_eq!(report.fid, 11);

        let expected_data: MessageData = [0xeb, 0x2f, 0x11, 0x8f, 0x7f, 0xf1, 0x00]
            .into_iter()
            .collect();
        assert_eq!(report.data, expected_data);
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
        assert!(!report.retransmit);
        assert_eq!(report.dac, 235);
        assert_eq!(report.fid, 10);

        let expected_data: MessageData = [0x44, 0x80, 0x10, 0x06, 0x80, 0x00, 0x00]
            .into_iter()
            .collect();
        assert_eq!(report.data, expected_data);
    }
}
