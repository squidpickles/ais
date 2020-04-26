use crate::errors::*;

pub fn parse_speed_over_ground(data: u16) -> Result<Option<f32>> {
    match data {
        0..=1022 => Ok(Some(data as f32 / 10.0)),
        1023 => Ok(None),
        _ => Err(format!("Invalid speed over ground: {}", data).into()),
    }
}

pub fn parse_longitude(data: i32) -> Result<Option<f32>> {
    match data {
        -108_000_000..=108_000_000 => Ok(Some(data as f32 / 600_000.0)),
        108_600_000 => Ok(None),
        _ => Err(format!("Invalid longitude: {}", data as f32 / 600_000.0).into()),
    }
}

pub fn parse_latitude(data: i32) -> Result<Option<f32>> {
    match data {
        -54_000_000..=54_000_000 => Ok(Some(data as f32 / 600_000.0)),
        54_600_000 => Ok(None),
        _ => Err(format!("Invalid latitude: {}", data as f32 / 600_000.0).into()),
    }
}

pub fn parse_cog(data: u16) -> Option<f32> {
    match data {
        3600 => None,
        _ => Some(data as f32 / 10.0),
    }
}

pub fn parse_heading(data: u16) -> Result<Option<u16>> {
    match data {
        0..=359 => Ok(Some(data)),
        511 => Ok(None),
        _ => Err(format!("Invalid heading: {}", data).into()),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Accuracy {
    Unaugmented,
    DGPS,
}

impl Accuracy {
    pub fn parse(data: u8) -> Result<Self> {
        match data {
            0 => Ok(Accuracy::Unaugmented),
            1 => Ok(Accuracy::DGPS),
            _ => Err("Unknown accuracy value".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RateOfTurn {
    raw: i8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Port,
    Starboard,
}

impl RateOfTurn {
    pub fn parse(data: u8) -> Option<Self> {
        #[allow(overflowing_literals)]
        match data as i8 {
            128 => None, // does indeed encode as 0x80
            -127..=127 => Some(RateOfTurn { raw: data as i8 }),
        }
    }

    pub fn rate(self) -> Option<f32> {
        match self.raw {
            -126..=126 => Some((self.raw as f32 / 4.733).powi(2)),
            -127 => None,
            127 => None,
            _ => unreachable!(),
        }
    }

    pub fn direction(self) -> Option<Direction> {
        match self.raw {
            0 => None,
            1..=127 => Some(Direction::Starboard),
            -127..=-1 => Some(Direction::Port),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ManeuverIndicator {
    NoSpecialManeuver,
    SpecialManeuver,
}
impl ManeuverIndicator {
    pub fn parse(data: u8) -> Result<Option<Self>> {
        #[allow(overflowing_literals)]
        match data as i8 {
            0 => Ok(None),
            1 => Ok(Some(Self::NoSpecialManeuver)),
            2 => Ok(Some(Self::SpecialManeuver)),
            _ => Err("Unknown maneuver indicator value".into()),
        }
    }
}
