//! Position Report (types 1-3)
use super::navigation::*;
use super::parsers::*;
use super::radio_status::{parse_radio, RadioStatus};
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct PositionReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub navigation_status: Option<NavigationStatus>,
    pub rate_of_turn: Option<RateOfTurn>,
    pub speed_over_ground: Option<f32>,
    pub position_accuracy: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub course_over_ground: Option<f32>,
    pub true_heading: Option<u16>,
    pub timestamp: u8,
    pub maneuver_indicator: Option<ManeuverIndicator>,
    pub raim: bool,
    pub radio_status: RadioStatus,
}

impl<'a> AisMessageType<'a> for PositionReport {
    fn name(&self) -> &'static str {
        "Position Report Class A"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], PositionReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, navigation_status) = map(take_bits(4u8), NavigationStatus::parse)(data)?;
        let (data, rate_of_turn) = map(take_bits(8u8), RateOfTurn::parse)(data)?;
        let (data, speed_over_ground) = map(take_bits(10u16), parse_speed_over_ground)(data)?;
        let (data, position_accuracy) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, course_over_ground) = map(take_bits(12u16), parse_cog)(data)?;
        let (data, true_heading) = map(take_bits(9u16), parse_heading)(data)?;
        let (data, timestamp) = take_bits(6u8)(data)?;
        let (data, maneuver_indicator) = map(take_bits(2u8), ManeuverIndicator::parse)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(3u8)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, radio_status) = parse_radio(data, message_type)?;
        Ok((
            data,
            PositionReport {
                message_type,
                repeat_indicator,
                mmsi,
                navigation_status,
                rate_of_turn,
                speed_over_ground,
                position_accuracy,
                longitude,
                latitude,
                course_over_ground,
                true_heading,
                timestamp,
                maneuver_indicator,
                raim,
                radio_status,
            },
        ))
    })(data)
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NavigationStatus {
    UnderWayUsingEngine,
    AtAnchor,
    NotUnderCommand,
    RestrictedManouverability,
    ConstrainedByDraught,
    Moored,
    Aground,
    EngagedInFishing,
    UnderWaySailing,
    ReservedForHSC,
    ReservedForWIG,
    Reserved01,
    Reserved02,
    Reserved03,
    AisSartIsActive,
    Unknown(u8),
}

impl NavigationStatus {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => Some(Self::UnderWayUsingEngine),
            1 => Some(Self::AtAnchor),
            2 => Some(Self::NotUnderCommand),
            3 => Some(Self::RestrictedManouverability),
            4 => Some(Self::ConstrainedByDraught),
            5 => Some(Self::Moored),
            6 => Some(Self::Aground),
            7 => Some(Self::EngagedInFishing),
            8 => Some(Self::UnderWaySailing),
            9 => Some(Self::ReservedForHSC),
            10 => Some(Self::ReservedForWIG),
            11 => Some(Self::Reserved01),
            12 => Some(Self::Reserved02),
            13 => Some(Self::Reserved03),
            14 => Some(Self::AisSartIsActive),
            15 => None,
            _ => Some(Self::Unknown(data)),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::messages::radio_status::{SubMessage, SyncState};
    use crate::test_helpers::*;

    #[test]
    fn test_position() {
        let bytestream = b"13u?etPv2;0n:dDPwUM1U1Cb069D";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(position.message_type, 1);
        assert_eq!(position.repeat_indicator, 0);
        assert_eq!(position.mmsi, 265547250);
        assert_eq!(
            position.navigation_status,
            Some(NavigationStatus::UnderWayUsingEngine)
        );
        let rate_of_turn = position.rate_of_turn.unwrap();
        f32_equal_naive(rate_of_turn.rate().unwrap().ceil(), 3.0);
        assert_eq!(rate_of_turn.direction(), Some(Direction::Port));
        assert_eq!(position.speed_over_ground, Some(13.9));
        assert_eq!(position.position_accuracy, Accuracy::Unaugmented);
        f32_equal_naive(position.longitude.unwrap().ceil(), 12.0);
        f32_equal_naive(position.latitude.unwrap().ceil(), 58.0);
        assert_eq!(position.course_over_ground, Some(40.4));
        assert_eq!(position.true_heading, Some(41));
        assert_eq!(position.timestamp, 53);
        assert_eq!(position.maneuver_indicator, None);
        assert!(!position.raim);
        if let RadioStatus::Sotdma(radio_status) = position.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 1);
            if let SubMessage::UtcHourAndMinute(hour, minute) = radio_status.sub_message {
                assert_eq!(hour, 17);
                assert_eq!(minute, 21);
            } else {
                panic!("Expected UTC Hour and Minute submessage");
            }
        } else {
            panic!("Expected SOTDMA message");
        }
    }

    #[test]
    fn test_type1() {
        let bytestream = b"16SteH0P00Jt63hHaa6SagvJ087r";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(bitstream.as_ref()).unwrap();
        f32_equal_naive(position.longitude.unwrap(), -70.7582);
        if let RadioStatus::Sotdma(radio_status) = position.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 2);
            assert_eq!(radio_status.sub_message, SubMessage::SlotNumber(506));
        } else {
            panic!("Expected SOTDMA message");
        }
    }

    #[test]
    fn test_type3() {
        let bytestream = b"38Id705000rRVJhE7cl9n;160000";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(position.message_type, 3);
        assert_eq!(position.mmsi, 563808000);
        assert_eq!(
            position.navigation_status.unwrap(),
            NavigationStatus::Moored
        );
        f32_equal_naive(position.longitude.unwrap(), -76.32753);
        f32_equal_naive(position.latitude.unwrap(), 36.91);
        f32_equal_naive(position.course_over_ground.unwrap(), 252.0);
        assert_eq!(position.true_heading.unwrap(), 352);
        assert_eq!(position.timestamp, 35);
        if let RadioStatus::Itdma(radio_status) = position.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_increment, 0);
            assert_eq!(radio_status.num_slots, 0);
            assert!(!radio_status.keep);
        } else {
            panic!("Expected ITDMA message");
        }
    }

    #[test]
    fn test_maneuver_indicator_out_of_spec() {
        let bytestream = b"33nQ:B50000FiEBRjpcK19qSR>`<";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(
            position.maneuver_indicator,
            Some(ManeuverIndicator::Unknown(3))
        );
    }
}
