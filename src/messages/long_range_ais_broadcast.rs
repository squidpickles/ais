//! Long Range AIS Broadcast message (type 27)
use super::navigation::*;
use super::parsers::*;
use super::position_report::NavigationStatus;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct LongRangeAisBroadcastMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub position_accuracy: Accuracy,
    pub raim: bool,
    pub navigation_status: Option<NavigationStatus>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub speed_over_ground: Option<f32>,
    pub course_over_ground: Option<f32>,
    pub gnss_position_status: bool,
}

impl<'a> AisMessageType<'a> for LongRangeAisBroadcastMessage {
    fn name(&self) -> &'static str {
        "Long Range AIS Broadcast message"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], LongRangeAisBroadcastMessage> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, position_accuracy) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, navigation_status) = map(take_bits(4u8), NavigationStatus::parse)(data)?;

        let (data, longitude) = map(
            |data| signed_i32(data, 18),
            |lon| {
                parse_longitude(lon).map(|val| {
                    if message_type == 27 {
                        val * 1000.0
                    } else {
                        val
                    }
                })
            },
        )(data)?;

        let (data, latitude) = map(
            |data| signed_i32(data, 17),
            |lat| {
                parse_latitude(lat).map(|val| {
                    if message_type == 27 {
                        val * 1000.0
                    } else {
                        val
                    }
                })
            },
        )(data)?;

        let (data, speed_over_ground) = map(take_bits(6u16), parse_speed_over_ground_62)(data)?;
        let (data, course_over_ground) = map(take_bits(9u16), parse_cog_511)(data)?;
        let (data, gnss_position_status) = map(take_bits(1u8), u8_to_bool)(data)?;

        Ok((
            data,
            LongRangeAisBroadcastMessage {
                message_type,
                repeat_indicator,
                mmsi,
                position_accuracy,
                raim,
                navigation_status,
                longitude,
                latitude,
                speed_over_ground,
                course_over_ground,
                gnss_position_status,
            },
        ))
    })(data)
}

/// Parse the speed over ground for Long Range AIS Broadcast Message (type 27)
fn parse_speed_over_ground_62(data: u16) -> Option<f32> {
    match data {
        63 => None,             // Speed not available
        _ => Some(data as f32), // Speed in knots (0-62)
    }
}

/// Parse the course over ground for Long Range AIS Broadcast Message (type 27)
fn parse_cog_511(data: u16) -> Option<f32> {
    match data {
        511 => None,            // Course not available
        _ => Some(data as f32), // Course in degrees (0-359)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use crate::test_helpers::f32_equal_naive;

    use super::*;
    // use crate::test_helpers::*;
    #[test]
    fn test_type27_example() {
        let bytestream = b"KC5E2b@U19PFdLbMuc5=ROv62<7m";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = LongRangeAisBroadcastMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 27);
        assert_eq!(report.repeat_indicator, 1);
        assert_eq!(report.mmsi, 206914217);
        assert_eq!(report.position_accuracy, Accuracy::Unaugmented);
        assert!(!report.raim);
        assert_eq!(
            report.navigation_status,
            Some(NavigationStatus::NotUnderCommand)
        );
        f32_equal_naive(report.longitude.unwrap(), 137.02333);
        f32_equal_naive(report.latitude.unwrap(), 4.84);
        assert_eq!(report.speed_over_ground, Some(57.0));
        assert_eq!(report.course_over_ground, Some(167.0));
        assert!(!report.gnss_position_status);
    }

    #[test]
    fn test_type27_signed_example() {
        let bytestream = b"K01;FQh?PbtE3P00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = LongRangeAisBroadcastMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 27);
        assert_eq!(report.mmsi, 1234567);
        f32_equal_naive(report.longitude.unwrap(), -13.368334);
        f32_equal_naive(report.latitude.unwrap(), -50.121665);
    }
}
