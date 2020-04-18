use crate::errors::*;

#[derive(Debug, PartialEq)]
pub enum EpfdType {
    Gps,
    Glonass,
    CombinedGpsAndGlonass,
    LoranC,
    Chayka,
    IntegratedNavigationSystem,
    Surveyed,
    Galileo,
}

impl EpfdType {
    pub fn parse(data: u8) -> Result<Option<Self>> {
        match data {
            0 => Ok(None),
            1 => Ok(Some(EpfdType::Gps)),
            2 => Ok(Some(EpfdType::Glonass)),
            3 => Ok(Some(EpfdType::CombinedGpsAndGlonass)),
            4 => Ok(Some(EpfdType::LoranC)),
            5 => Ok(Some(EpfdType::Chayka)),
            6 => Ok(Some(EpfdType::IntegratedNavigationSystem)),
            7 => Ok(Some(EpfdType::Surveyed)),
            8 => Ok(Some(EpfdType::Galileo)),
            15 => Ok(None),
            _ => Err(format!("Unknown Epfd type: {}", data).into()),
        }
    }
}
