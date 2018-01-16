use errors::*;
use nom::IResult;
use super::{AisMessage, BitStream};
use super::radio_status::{ItdmaMessage, RadioStatus, SotdmaMessage};

#[derive(Debug)]
pub struct PositionReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub navigation_status: Option<NavigationStatus>,
    pub rate_of_turn: Option<RateOfTurn>,
    pub speed_over_ground: Option<f32>,
    pub position_accuracy: Accuracy,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub course_over_ground: Option<f32>,
    pub true_heading: Option<u16>,
    pub timestamp: u8,
    pub maneuver_indicator: Option<ManeuverIndicator>,
    pub raim: bool,
    pub radio_status: RadioStatus,
}

impl<'a> AisMessage<'a> for PositionReport {
    fn name(&self) -> &'static str {
        "Position Report Class A"
    }

    fn parse(data: BitStream) -> Result<Self> {
        match position_parser(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
            IResult::Incomplete(_) => Err("incomplete AIS sentence")?,
        }
    }
}

fn parse_speed_over_ground(data: u16) -> Result<Option<f32>> {
    match data {
        0...1022 => Ok(Some(data as f32 / 10.0)),
        1023 => Ok(None),
        _ => Err(format!("Invalid speed over ground: {}", data).into()),
    }
}

fn parse_longitude(data: u32) -> Result<Option<f64>> {
    match data {
        0...108000000 => Ok(Some(data as f64 / 600000.0)),
        108600000 => Ok(None),
        _ => Err(format!("Invalid longitude: {}", data as f64 / 600000.0).into()),
    }
}

fn parse_latitude(data: u32) -> Result<Option<f64>> {
    match data {
        0...54000000 => Ok(Some(data as f64 / 600000.0)),
        54600000 => Ok(None),
        _ => Err(format!("Invalid latitude: {}", data as f64 / 600000.0).into()),
    }
}

fn parse_cog(data: u16) -> Option<f32> {
    match data {
        3600 => None,
        _ => Some(data as f32 / 10.0),
    }
}

fn parse_heading(data: u16) -> Result<Option<u16>> {
    match data {
        0...359 => Ok(Some(data)),
        511 => Ok(None),
        _ => Err(format!("Invalid heading: {}", data).into()),
    }
}

fn parse_raim(data: u8) -> Result<bool> {
    match data {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("Invalid RAIM status: {}", data).into()),
    }
}

fn parse_radio(input: (&[u8], usize), msg_type: u8) -> IResult<(&[u8], usize), RadioStatus> {
    match msg_type {
        1 | 2 => SotdmaMessage::parser(input),
        3 => ItdmaMessage::parser(input),
        _ => ::nom::IResult::Error(::nom::ErrorKind::Digit),
    }
}

named!(
    position_parser<PositionReport>,
    bits!(do_parse!(
        msg_type: take_bits!(u8, 6) >> repeat: take_bits!(u8, 2) >> mmsi: take_bits!(u32, 30)
            >> nav_status: map_res!(take_bits!(u8, 4), NavigationStatus::parse)
            >> rot: take_bits!(u8, 8)
            >> sog: map_res!(take_bits!(u16, 10), parse_speed_over_ground)
            >> accuracy: map_res!(take_bits!(u8, 1), Accuracy::parse)
            >> lon: map_res!(take_bits!(u32, 28), parse_longitude)
            >> lat: map_res!(take_bits!(u32, 27), parse_latitude)
            >> cog: take_bits!(u16, 12) >> hdg: map_res!(take_bits!(u16, 9), parse_heading)
            >> stamp: take_bits!(u8, 6) >> maneuver: take_bits!(u8, 2)
            >> spare: take_bits!(u8, 3) >> raim: map_res!(take_bits!(u8, 1), parse_raim)
            >> radio: apply!(parse_radio, msg_type) >> (PositionReport {
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
pub enum Accuracy {
    Unaugmented,
    DGPS,
}

impl Accuracy {
    fn parse(data: u8) -> Result<Self> {
        match data {
            0 => Ok(Accuracy::Unaugmented),
            1 => Ok(Accuracy::DGPS),
            _ => Err("Unknown accuracy value".into()),
        }
    }
}

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

#[derive(Debug)]
pub struct RateOfTurn {
    raw: i8,
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Port,
    Starboard,
}

impl RateOfTurn {
    pub fn parse(data: u8) -> Option<Self> {
        #[allow(overflowing_literals)]
        match data as i8 {
            128 => None, // does indeed encode as 0x80
            -127...127 => Some(RateOfTurn { raw: data as i8 }),
            _ => unreachable!(),
        }
    }

    pub fn rate(&self) -> Option<f32> {
        match self.raw {
            -126...126 => Some((self.raw as f32 / 4.733).powi(2)),
            -127 => None,
            127 => None,
            _ => unreachable!(),
        }
    }

    pub fn direction(&self) -> Option<Direction> {
        match self.raw {
            0 => None,
            1...127 => Some(Direction::Starboard),
            -127...-1 => Some(Direction::Port),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ManeuverIndicator {
    NoSpecialManeuver,
    SpecialManeuver,
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::radio_status::{SubMessage, SyncState};

    #[test]
    fn test_position() {
        let bytestream = b"13u?etPv2;0n:dDPwUM1U1Cb069D";
        let bitstream = ::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(&bitstream).unwrap();
        assert_eq!(position.message_type, 1);
        assert_eq!(position.repeat_indicator, 0);
        assert_eq!(position.mmsi, 265547250);
        assert_eq!(
            position.navigation_status,
            Some(NavigationStatus::UnderWayUsingEngine)
        );
        let rate_of_turn = position.rate_of_turn.unwrap();
        assert_eq!(rate_of_turn.rate().unwrap().ceil(), 3.0);
        assert_eq!(rate_of_turn.direction(), Some(Direction::Port));
        assert_eq!(position.speed_over_ground, Some(13.9));
        assert_eq!(position.position_accuracy, Accuracy::Unaugmented);
        assert_eq!(position.longitude.unwrap().ceil(), 12.0);
        assert_eq!(position.latitude.unwrap().ceil(), 58.0);
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
    fn test_type3() {
        // FIXME: broken test
        let bytestream = b"38Id705000rRVJhE7cl9n;160000";
        let bitstream = ::messages::unarmor(bytestream, 0).unwrap();
        let position = PositionReport::parse(&bitstream).unwrap();
    }
}
