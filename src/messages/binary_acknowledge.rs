//! Binary Acknowledge (type 7)
#[cfg(all(not(feature = "std"), not(feature = "alloc")))] 
 use super::nom_noalloc::many_m_n; 
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
#[cfg(any(feature = "std", feature = "alloc"))] 
use nom::multi::many_m_n;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct Acknowledgement {
    pub mmsi: u32,
    pub seq_num: u8,
}

impl Acknowledgement {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, seq_num) = take_bits(2u8)(data)?;
        Ok((data, Self { mmsi, seq_num }))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub type AcknowledgementList = lib::std::vec::Vec<Acknowledgement>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type AcknowledgementList = lib::std::vec::Vec<Acknowledgement, 4>;

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryAcknowledge {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub acks: AcknowledgementList,
}

impl<'a> AisMessageType<'a> for BinaryAcknowledge {
    fn name(&self) -> &'static str {
        "Binary Acknowledge"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base<'a>(data: &'a [u8]) -> IResult<&'a [u8], BinaryAcknowledge> {
    bits(move |data: (&'a [u8], usize)| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;

        #[cfg(any(feature = "std", feature = "alloc"))]
        let (data, acks) = many_m_n(1, 4, Acknowledgement::parse)(data)?;
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let (data, acks) = many_m_n::<_, _, _, _, 4>(1, Acknowledgement::parse)(data)?;

        Ok((
            data,
            BinaryAcknowledge {
                message_type,
                repeat_indicator,
                mmsi,
                acks,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;

    #[test]
    fn test_type7_example_1() {
        let bytestream = b"702R5`hwCt40";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = BinaryAcknowledge::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 7);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2655651);
        assert_eq!(report.acks.len(), 1);
        let ack = &report.acks[0];
        assert_eq!(ack.mmsi, 265547840);
        assert_eq!(ack.seq_num, 0);
    }

    #[test]
    fn test_type7_example_2() {
        let bytestream = b"702R5`hwCjq8";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = BinaryAcknowledge::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 7);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2655651);
        assert_eq!(report.acks.len(), 1);
        let ack = &report.acks[0];
        assert_eq!(ack.mmsi, 265538450);
        assert_eq!(ack.seq_num, 0);

        // Ensure other MMSIs are not present
        assert!(report.acks.get(1).is_none());
        assert!(report.acks.get(2).is_none());
        assert!(report.acks.get(3).is_none());
    }
}
