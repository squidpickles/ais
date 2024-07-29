//! Data Link Management Message (type 20)
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
pub struct SlotReservation {
    pub offset: u16,
    pub num_slots: u8,
    pub timeout: u8,
    pub increment: u16,
}

impl SlotReservation {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, offset) = take_bits(12u16)(data)?;
        let (data, num_slots) = take_bits(4u8)(data)?;
        let (data, timeout) = take_bits(3u8)(data)?;
        let (data, increment) = take_bits(11u16)(data)?;
        Ok((
            data,
            Self {
                offset,
                num_slots,
                timeout,
                increment,
            },
        ))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub type SlotReservationList = lib::std::vec::Vec<SlotReservation>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type SlotReservationList = lib::std::vec::Vec<SlotReservation, 4>;

#[derive(Debug, PartialEq, Eq)]
pub struct DataLinkManagementMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub reservations: SlotReservationList,
}

impl<'a> AisMessageType<'a> for DataLinkManagementMessage {
    fn name(&self) -> &'static str {
        "Data Link Management Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base<'a>(data: &'a [u8]) -> IResult<&'a [u8], DataLinkManagementMessage> {
    bits(move |data: (&'a [u8], usize)| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        let (data, reservations) = many_m_n(1, 4, SlotReservation::parse)(data)?;
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let (data, reservations) = many_m_n::<_, _, _, _, 4>(1, SlotReservation::parse)(data)?;
        Ok((
            data,
            DataLinkManagementMessage {
                message_type,
                repeat_indicator,
                mmsi,
                reservations,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;

    #[test]
    fn test_2_slots() {
        let bytestream = b"D02<HjiUHBfr<`E6D0";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = DataLinkManagementMessage::parse(bitstream.as_ref()).unwrap();
        assert_eq!(report.message_type, 20);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2300107);
        assert_eq!(report.reservations.len(), 2);
        assert_eq!(report.reservations[0].num_slots, 1);
        assert_eq!(report.reservations[1].increment, 1125);
    }

    #[test]
    fn test_3_slots() {
        let bytestream = b"D02;bK0RlLfq6DM6DA8u6D0";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = DataLinkManagementMessage::parse(bitstream.as_ref()).unwrap();
        assert_eq!(report.message_type, 20);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2288236);
        assert_eq!(report.reservations.len(), 3);
        assert_eq!(report.reservations[1].num_slots, 1);
        assert_eq!(report.reservations[2].increment, 1125);
    }
}
