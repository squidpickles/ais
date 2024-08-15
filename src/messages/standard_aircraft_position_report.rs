//! Standard SAR Aircraft Position Report (type 9)
use super::navigation::*;
use super::parsers::*;
use super::radio_status::*;

use super::types::AssignedMode;
use super::types::Dte;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct SARPositionReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub altitude: Option<u16>,
    pub speed_over_ground: Option<f32>,
    pub position_accuracy: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub course_over_ground: Option<f32>,
    pub timestamp: u8,
    pub dte: Dte,
    pub assigned_mode: AssignedMode,
    pub raim: bool,
    pub radio_status: RadioStatus,
}

impl<'a> AisMessageType<'a> for SARPositionReport {
    fn name(&self) -> &'static str {
        "Standard SAR Aircraft Position Report"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], SARPositionReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, altitude) = map(take_bits(12u16), parse_altitude)(data)?;
        let (data, speed_over_ground) = map(take_bits(10u16), parse_speed_over_ground_sar)(data)?;
        let (data, position_accuracy) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, course_over_ground) = map(take_bits(12u16), parse_cog)(data)?;
        let (data, timestamp) = take_bits(6u8)(data)?;
        let (data, _regional_reserved) = take_bits::<_, u8, _, _>(8u8)(data)?;
        let (data, dte) = map(take_bits::<_, u8, _, _>(1u8), Into::into)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(3u8)(data)?;
        let (data, assigned_mode) = map(take_bits(1u8), AssignedMode::parse)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, radio_status) = parse_radio(data, message_type)?;

        Ok((
            data,
            SARPositionReport {
                message_type,
                repeat_indicator,
                mmsi,
                altitude,
                speed_over_ground,
                position_accuracy,
                longitude,
                latitude,
                course_over_ground,
                timestamp,
                dte,
                assigned_mode,
                raim,
                radio_status,
            },
        ))
    })(data)
}

/// Parse the altitude field
fn parse_altitude(data: u16) -> Option<u16> {
    match data {
        4095 => None,       // Altitude not available
        4094 => Some(4094), // 4094 meters or higher
        _ => Some(data),
    }
}

/// Parse the speed over ground for SAR Position Report (type 9)
fn parse_speed_over_ground_sar(data: u16) -> Option<f32> {
    match data {
        1023 => None,         // Speed not available
        1022 => Some(1022.0), // 1022 knots or higher
        _ => Some(data as f32),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::messages::radio_status::{SubMessage, SyncState};
    use crate::test_helpers::*;

    #[test]
    fn test_type9_example() {
        let bytestream = b"91b55wi;hbOS@OdQAC062Ch2089h";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = SARPositionReport::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 9);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 111232511);
        assert_eq!(report.altitude, Some(303));
        assert_eq!(report.speed_over_ground, Some(42.0));
        assert_eq!(report.position_accuracy, Accuracy::Unaugmented);
        f32_equal_naive(report.longitude.unwrap(), -6.2788434);
        f32_equal_naive(report.latitude.unwrap(), 58.144);
        assert_eq!(report.course_over_ground, Some(154.5));
        assert_eq!(report.timestamp, 15);
        assert_eq!(report.dte, Dte::NotReady);
        if let RadioStatus::Sotdma(radio_status) = report.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 1);
            if let SubMessage::UtcHourAndMinute(hour, minute) = radio_status.sub_message {
                assert_eq!(hour, 0);
                assert_eq!(minute, 14);
            } else {
                panic!("Expected UTC Hour and Minute submessage");
            }
        } else {
            panic!("Expected SOTDMA message");
        }
        assert!(!report.raim);
    }
}
