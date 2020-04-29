pub fn parse_speed_over_ground(data: u16) -> Option<f32> {
    match data {
        1023 => None,
        _ => Some(data as f32 / 10.0),
    }
}

pub fn parse_longitude(data: i32) -> Option<f32> {
    match data {
        108_600_000 => None,
        _ => Some(data as f32 / 600_000.0),
    }
}

pub fn parse_latitude(data: i32) -> Option<f32> {
    match data {
        54_600_000 => None,
        _ => Some(data as f32 / 600_000.0),
    }
}

pub fn parse_cog(data: u16) -> Option<f32> {
    match data {
        3600 => None,
        _ => Some(data as f32 / 10.0),
    }
}

pub fn parse_heading(data: u16) -> Option<u16> {
    match data {
        511 => None,
        _ => Some(data),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Accuracy {
    Unaugmented,
    DGPS,
}

impl Accuracy {
    pub fn parse(data: u8) -> Self {
        match data {
            0 => Self::Unaugmented,
            1 => Self::DGPS,
            _ => unreachable!(),
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
    Unknown(u8),
}
impl ManeuverIndicator {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => None,
            1 => Some(Self::NoSpecialManeuver),
            2 => Some(Self::SpecialManeuver),
            _ => Some(Self::Unknown(data)),
        }
    }
}
