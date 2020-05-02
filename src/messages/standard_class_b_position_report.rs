//! Position Report (types 1-3)
use super::navigation::*;
use super::parsers::*;
use super::radio_status::{ItdmaMessage, RadioStatus, SotdmaMessage};
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct StandardClassBPositionReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub speed_over_ground: Option<f32>,
    pub position_accuracy: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub course_over_ground: Option<f32>,
    pub true_heading: Option<u16>,
    pub timestamp: u8,
    pub cs_unit: CarrierSense,
    pub has_display: bool,
    pub has_dsc: bool,
    pub whole_band: bool,
    pub accepts_message_22: bool,
    pub assigned_mode: AssignedMode,
    pub raim: bool,
    pub radio_status: RadioStatus,
}

impl<'a> AisMessageType<'a> for StandardClassBPositionReport {
    fn name(&self) -> &'static str {
        "Standard Class B Position Report"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

#[derive(Debug, PartialEq)]
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
            _ => panic!("Expected 1 bit value here"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AssignedMode {
    Autonomous,
    Assigned,
}

impl AssignedMode {
    pub fn parse(val: u8) -> Self {
        match val {
            0 => Self::Autonomous,
            1 => Self::Assigned,
            _ => panic!("Expected 1 bit value here"),
        }
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], StandardClassBPositionReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, repeat_indicator) = take_bits::<_, _, _, (_, _)>(2u8)(data)?;
        let (data, mmsi) = take_bits::<_, _, _, (_, _)>(30u32)(data)?;
        let (data, _regional_reserved) = take_bits::<_, u8, _, (_, _)>(8u8)(data)?;
        let (data, speed_over_ground) =
            map(take_bits::<_, _, _, (_, _)>(10u16), parse_speed_over_ground)(data)?;
        let (data, position_accuracy) =
            map(take_bits::<_, _, _, (_, _)>(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, course_over_ground) = map(take_bits::<_, _, _, (_, _)>(12u16), parse_cog)(data)?;
        let (data, true_heading) = map(take_bits::<_, _, _, (_, _)>(9u16), parse_heading)(data)?;
        let (data, timestamp) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, _regional_reserved) = take_bits::<_, u8, _, (_, _)>(2u8)(data)?;
        let (data, cs_unit) = map(take_bits::<_, _, _, (_, _)>(1u8), CarrierSense::parse)(data)?;
        let (data, has_display) = map(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, has_dsc) = map(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, whole_band) = map(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, accepts_message_22) = map(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, assigned_mode) =
            map(take_bits::<_, _, _, (_, _)>(1u8), AssignedMode::parse)(data)?;
        let (data, raim) = map(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, cs_selector) = take_bits::<_, _, _, (_, _)>(1u8)(data)?;
        let (data, radio_status) = match cs_selector {
            0 => SotdmaMessage::parse(data)?,
            1 => ItdmaMessage::parse(data)?,
            _ => panic!("Single bit value expected"),
        };
        Ok((
            data,
            StandardClassBPositionReport {
                message_type,
                repeat_indicator,
                mmsi,
                speed_over_ground,
                position_accuracy,
                longitude,
                latitude,
                course_over_ground,
                true_heading,
                timestamp,
                cs_unit,
                has_display,
                has_dsc,
                whole_band,
                accepts_message_22,
                assigned_mode,
                raim,
                radio_status,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::messages::radio_status::{SubMessage, SyncState};
    use crate::test_helpers::*;

    #[test]
    fn test_position() {
        let bytestream = b"B6:hQDh0029Pt<4TAS003h6TSP00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = StandardClassBPositionReport::parse(&bitstream).unwrap();
        assert_eq!(report.message_type, 18);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 413933907);
        assert_eq!(report.speed_over_ground, Some(0.0));
        assert_eq!(report.position_accuracy, Accuracy::Unaugmented);
        f32_equal_naive(report.longitude.unwrap(), 120.16217);
        f32_equal_naive(report.latitude.unwrap(), 31.924133);
        assert_eq!(report.course_over_ground, Some(0.0));
        assert_eq!(report.true_heading, Some(480));
        assert_eq!(report.timestamp, 13);
        assert_eq!(report.cs_unit, CarrierSense::CarrierSense);
        assert_eq!(report.has_display, false);
        assert_eq!(report.has_dsc, false);
        assert_eq!(report.whole_band, true);
        assert_eq!(report.assigned_mode, AssignedMode::Autonomous);
        assert_eq!(report.raim, false);
        if let RadioStatus::Itdma(radio_status) = report.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::NumberOfReceivedStations);
            assert_eq!(radio_status.slot_increment, 0);
            assert_eq!(radio_status.num_slots, 0);
            assert_eq!(radio_status.keep, false);
        } else {
            panic!("Expected SOTDMA message");
        }
    }
}
