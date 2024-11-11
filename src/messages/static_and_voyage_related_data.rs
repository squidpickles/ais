//! Static and Voyage Related Data (type 5)
use super::parsers::*;
use super::types::*;
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StaticAndVoyageRelatedData {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub ais_version: u8,
    pub imo_number: u32,
    pub callsign: AsciiString,
    pub vessel_name: AsciiString,
    pub ship_type: Option<ShipType>,
    pub dimension_to_bow: u16,
    pub dimension_to_stern: u16,
    pub dimension_to_port: u16,
    pub dimension_to_starboard: u16,
    pub epfd_type: Option<EpfdType>,
    pub eta_month_utc: Option<u8>,
    pub eta_day_utc: Option<u8>,
    pub eta_hour_utc: u8,
    pub eta_minute_utc: Option<u8>,
    pub draught: f32,
    pub destination: AsciiString,
    pub dte: Dte,
}

impl<'a> AisMessageType<'a> for StaticAndVoyageRelatedData {
    fn name(&self) -> &'static str {
        "Static and Voyage Related Data"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_message(data)?;
        Ok(report)
    }
}

fn parse_message(data: &[u8]) -> IResult<&[u8], StaticAndVoyageRelatedData> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, ais_version) = take_bits(2u8)(data)?;
        let (data, imo_number) = take_bits(30u32)(data)?;
        let (data, callsign) = parse_6bit_ascii(data, 42)?;
        let (data, vessel_name) = parse_6bit_ascii(data, 120)?;
        let (data, ship_type) = map(take_bits(8u8), ShipType::parse)(data)?;
        let (data, dimension_to_bow) = take_bits(9u16)(data)?;
        let (data, dimension_to_stern) = take_bits(9u16)(data)?;
        let (data, dimension_to_port) = take_bits(6u16)(data)?;
        let (data, dimension_to_starboard) = take_bits(6u16)(data)?;
        let (data, epfd_type) = map(take_bits(4u8), EpfdType::parse)(data)?;
        let (data, eta_month_utc) = parse_month(data)?;
        let (data, eta_day_utc) = parse_day(data)?;
        let (data, eta_hour_utc) = parse_hour(data)?;
        let (data, eta_minute_utc) = parse_minsec(data)?;
        let (data, draught) = map(take_bits::<_, u8, _, _>(8u8), |raw_draught| {
            raw_draught as f32 / 10.0
        })(data)?;
        // Sometimes these messages are truncated.
        // First, take only up to 120 bits for destination
        let (data, destination) =
            parse_6bit_ascii(data, lib::std::cmp::min(120, remaining_bits(data)))?;
        // Second, if there are no bits left for DTE, use the default value
        let (data, dte) = if remaining_bits(data) > 0 {
            map(take_bits::<_, u8, _, _>(1u8), Into::into)(data)?
        } else {
            (data, Dte::default())
        };
        // Finally, take a last bit if there's still room, so the returned data doesn't have leftovers
        let (data, _) = if remaining_bits(data) > 0 {
            take_bits(1u8)(data)?
        } else {
            (data, 0u8)
        };
        Ok((
            data,
            StaticAndVoyageRelatedData {
                message_type,
                repeat_indicator,
                mmsi,
                ais_version,
                imo_number,
                callsign,
                vessel_name,
                ship_type,
                dimension_to_bow,
                dimension_to_stern,
                dimension_to_port,
                dimension_to_starboard,
                epfd_type,
                eta_month_utc,
                eta_day_utc,
                eta_hour_utc,
                eta_minute_utc,
                draught,
                destination,
                dte,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::test_helpers::f32_equal_naive;

    #[test]
    fn test_type5_truncated() {
        let bytestream = b"5341U9`00000uCGCKL0u=@T4000000000000001?<@<47u;b004Sm51DQ0C@";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = StaticAndVoyageRelatedData::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 5);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 205546790);
        assert_eq!(message.callsign, "OT5467");
        assert_eq!(
            message.ship_type,
            Some(ShipType::CargoNoAdditionalInformation)
        );
        assert_eq!(message.eta_month_utc, Some(4));
        assert_eq!(message.destination, "ROTTERDAM");
        assert_eq!(message.epfd_type, None);
        assert_eq!(message.dte, Dte::Ready);
    }

    #[test]
    fn test_type5_multipart() {
        /*
        !AIVDM,2,1,2,A,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP0,0*78
        !AIVDM,2,2,2,A,00000000000,2*26
        TODO: use parser here
        */
        let bytestream = b"53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP000000000000";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = StaticAndVoyageRelatedData::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 5);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 244250440);
        assert_eq!(message.callsign, "PF8793");
        assert_eq!(message.ship_type, Some(ShipType::PleasureCraft));
        f32_equal_naive(message.draught, 2.1);
        assert_eq!(message.eta_month_utc, Some(1));
        assert_eq!(message.destination, "NL LMMR");
        assert_eq!(message.epfd_type, None);
        assert_eq!(message.dte, Dte::Ready);
    }
}
