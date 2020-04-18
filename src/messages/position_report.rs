use super::navigation::*;
use super::radio_status::{parse_radio, RadioStatus};
use super::{signed_i32, u8_to_bool, AisMessageType, BitStream};
use crate::errors::*;
use nom::*;

#[derive(Debug)]
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

    fn parse(data: BitStream) -> Result<Self> {
        match position_parser(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
            IResult::Incomplete(_) => Err("incomplete AIS sentence".into()),
        }
    }
}

named!(
    position_parser<PositionReport>,
    bits!(do_parse!(
        msg_type: take_bits!(u8, 6)
            >> repeat: take_bits!(u8, 2)
            >> mmsi: take_bits!(u32, 30)
            >> nav_status: map_res!(take_bits!(u8, 4), NavigationStatus::parse)
            >> rot: take_bits!(u8, 8)
            >> sog: map_res!(take_bits!(u16, 10), parse_speed_over_ground)
            >> accuracy: map_res!(take_bits!(u8, 1), Accuracy::parse)
            >> lon: map_res!(apply!(signed_i32, 28), parse_longitude)
            >> lat: map_res!(apply!(signed_i32, 27), parse_latitude)
            >> cog: take_bits!(u16, 12)
            >> hdg: map_res!(take_bits!(u16, 9), parse_heading)
            >> stamp: take_bits!(u8, 6)
            >> maneuver: take_bits!(u8, 2)
            >> spare: take_bits!(u8, 3)
            >> raim: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> radio: apply!(parse_radio, msg_type)
            >> (PositionReport {
                message_type: msg_type,
                repeat_indicator: repeat,
                mmsi: mmsi,
                navigation_status: nav_status,
                rate_of_turn: RateOfTurn::parse(rot),
                speed_over_ground: sog,
                position_accuracy: accuracy,
                longitude: lon,
                latitude: lat,
                course_over_ground: parse_cog(cog),
                true_heading: hdg,
                timestamp: stamp,
                maneuver_indicator: None,
                raim: raim,
                radio_status: radio,
            })
    ))
);

#[derive(Debug, PartialEq)]
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
}

impl NavigationStatus {
    fn parse(data: u8) -> Result<Option<Self>> {
        match data {
            0 => Ok(Some(NavigationStatus::UnderWayUsingEngine)),
            1 => Ok(Some(NavigationStatus::AtAnchor)),
            2 => Ok(Some(NavigationStatus::NotUnderCommand)),
            3 => Ok(Some(NavigationStatus::RestrictedManouverability)),
            4 => Ok(Some(NavigationStatus::ConstrainedByDraught)),
            5 => Ok(Some(NavigationStatus::Moored)),
            6 => Ok(Some(NavigationStatus::Aground)),
            7 => Ok(Some(NavigationStatus::EngagedInFishing)),
            8 => Ok(Some(NavigationStatus::UnderWaySailing)),
            9 => Ok(Some(NavigationStatus::ReservedForHSC)),
            10 => Ok(Some(NavigationStatus::ReservedForWIG)),
            11 => Ok(Some(NavigationStatus::Reserved01)),
            12 => Ok(Some(NavigationStatus::Reserved02)),
            13 => Ok(Some(NavigationStatus::Reserved03)),
            14 => Ok(Some(NavigationStatus::AisSartIsActive)),
            15 => Ok(None),
            _ => Err(format!("Unknown navigation status: {}", data).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::super::super::test_helpers::*;
    use super::*;
    use crate::messages::radio_status::{SubMessage, SyncState};

    #[test]
    fn test_position() {
        let bytestream = b"13u?etPv2;0n:dDPwUM1U1Cb069D";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(&bitstream).unwrap();
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
        assert_eq!(position.raim, false);
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
        let position = PositionReport::parse(&bitstream).unwrap();
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
        // FIXME: broken test
        let bytestream = b"38Id705000rRVJhE7cl9n;160000";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(&bitstream).unwrap();
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
            assert_eq!(radio_status.keep, false);
        } else {
            panic!("Expected ITDMA message");
        }
    }
}
