use errors::*;
use nom::IResult;
use super::{AisMessage, BitStream};

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
    pub radio_status: u32,
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
            >> radio: take_bits!(u32, 19) >> (PositionReport {
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

pub struct RateOfTurn {
    raw: i8,
}

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

pub enum ManeuverIndicator {
    NoSpecialManeuver,
    SpecialManeuver,
}
