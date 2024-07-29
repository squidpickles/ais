//! Binary Broadcast Message (type 8)
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
use nom::IResult;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
const MAX_DATA_SIZE_BYTES: usize = 119;

#[cfg(any(feature = "std", feature = "alloc"))]
pub type MessageData = lib::std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type MessageData = lib::std::vec::Vec<u8, MAX_DATA_SIZE_BYTES>;

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryBroadcastMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    /// Designated area code
    pub dac: u16,
    /// Functional ID
    pub fid: u8,
    pub data: MessageData,
}

impl<'a> AisMessageType<'a> for BinaryBroadcastMessage {
    fn name(&self) -> &'static str {
        "Binary Broadcast Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CarrierSense {
    /// Class B SOTDMA unit
    Sotdma,
    /// Class B Carrier Sense unit
    CarrierSense,
}

impl CarrierSense {
    pub fn parse(val: u8) -> Self {
        match val {
            0 => Self::Sotdma,
            1 => Self::CarrierSense,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignedMode {
    Autonomous,
    Assigned,
}

impl AssignedMode {
    pub fn parse(val: u8) -> Self {
        match val {
            0 => Self::Autonomous,
            1 => Self::Assigned,
            _ => unreachable!(),
        }
    }
}

fn parse_base<'a>(data: &'a [u8]) -> IResult<&'a [u8], BinaryBroadcastMessage> {
    bits(move |data: (&'a [u8], usize)| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
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
            BinaryBroadcastMessage {
                message_type,
                repeat_indicator,
                mmsi,
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
    fn test_meteorological_hydrological_old() {
        // !AIVDM,1,1,,A,8@2<HW@0BkdhF0dcH5R`Q@kDJjD;WwfRwwwwwwwwwwwwwwwwwwwwwwwwwt0,2*60
        let bytestream = b"8@2<HW@0BkdhF0dcH5R`Q@kDJjD;WwfRwwwwwwwwwwwwwwwwwwwwwwwwwt0";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = BinaryBroadcastMessage::parse(bitstream.as_ref()).unwrap();
        assert_eq!(report.message_type, 8);
        assert_eq!(report.repeat_indicator, 1);
        assert_eq!(report.mmsi, 2300061);
        assert_eq!(report.dac, 1);
        assert_eq!(report.fid, 11);
    }

    #[test]
    fn test_meteorological_hydrological_new() {
        // !AIVDM,1,1,,A,8@2R5Ph0GhEa?1bGBviEOwvlFR06EuOwgqriwnSwe7wvlOwwsAwwnSGmwvwt,0*64
        let bytestream = b"8@2R5Ph0GhEa?1bGBviEOwvlFR06EuOwgqriwnSwe7wvlOwwsAwwnSGmwvwt";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = BinaryBroadcastMessage::parse(bitstream.as_ref()).unwrap();
        assert_eq!(report.message_type, 8);
        assert_eq!(report.repeat_indicator, 1);
        assert_eq!(report.mmsi, 2655619);
        assert_eq!(report.dac, 1);
        assert_eq!(report.fid, 31);
    }
}
