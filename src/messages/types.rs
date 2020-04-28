//! Common data types
use crate::errors::*;

#[derive(Debug, PartialEq, Copy, Clone)]
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

#[derive(Debug, PartialEq)]
pub enum ShipType {
    Reserved,
    WingInGround,
    WingInGroundHazardousCategoryA,
    WingInGroundHazardousCategoryB,
    WingInGroundHazardousCategoryC,
    WingInGroundHazardousCategoryD,
    WingInGroundReserved,
    Fishing,
    Towing,
    TowingLarge,
    Dredging,
    DivingOps,
    MilitaryOps,
    Sailing,
    PleasureCraft,
    HighSpeedCraft,
    HighSpeedCraftHazardousCategoryA,
    HighSpeedCraftHazardousCategoryB,
    HighSpeedCraftHazardousCategoryC,
    HighSpeedCraftHazardousCategoryD,
    HighSpeedCraftReserved,
    HighSpeedCraftNoAdditionalInformation,
    PilotVessel,
    SearchAndRescueVessel,
    Tug,
    PortTender,
    AntiPollutionEquipment,
    LawEnforcement,
    SpareLocalVessel,
    MedicalTransport,
    NoncombatantShip,
    Passenger,
    PassengerHazardousCategoryA,
    PassengerHazardousCategoryB,
    PassengerHazardousCategoryC,
    PassengerHazardousCategoryD,
    PassengerReserved,
    PassengerNoAdditionalInformation,
    Cargo,
    CargoHazardousCategoryA,
    CargoHazardousCategoryB,
    CargoHazardousCategoryC,
    CargoHazardousCategoryD,
    CargoReserved,
    CargoNoAdditionalInformation,
    Tanker,
    TankerHazardousCategoryA,
    TankerHazardousCategoryB,
    TankerHazardousCategoryC,
    TankerHazardousCategoryD,
    TankerReserved,
    TankerNoAdditionalInformation,
    Other,
    OtherHazardousCategoryA,
    OtherHazardousCategoryB,
    OtherHazardousCategoryC,
    OtherHazardousCategoryD,
    OtherReserved,
    OtherNoAdditionalInformation,
}

impl ShipType {
    pub fn parse(data: u8) -> Result<Option<Self>> {
        match data {
            0 => Ok(None),
            1..=19 => Ok(Some(Self::Reserved)),
            20 => Ok(Some(Self::WingInGround)),
            21 => Ok(Some(Self::WingInGroundHazardousCategoryA)),
            22 => Ok(Some(Self::WingInGroundHazardousCategoryB)),
            23 => Ok(Some(Self::WingInGroundHazardousCategoryC)),
            24 => Ok(Some(Self::WingInGroundHazardousCategoryD)),
            25..=29 => Ok(Some(Self::WingInGroundReserved)),
            30 => Ok(Some(Self::Fishing)),
            31 => Ok(Some(Self::Towing)),
            32 => Ok(Some(Self::TowingLarge)),
            33 => Ok(Some(Self::Dredging)),
            34 => Ok(Some(Self::DivingOps)),
            35 => Ok(Some(Self::MilitaryOps)),
            36 => Ok(Some(Self::Sailing)),
            37 => Ok(Some(Self::PleasureCraft)),
            38..=39 => Ok(Some(Self::Reserved)),
            40 => Ok(Some(Self::HighSpeedCraft)),
            41 => Ok(Some(Self::HighSpeedCraftHazardousCategoryA)),
            42 => Ok(Some(Self::HighSpeedCraftHazardousCategoryB)),
            43 => Ok(Some(Self::HighSpeedCraftHazardousCategoryC)),
            44 => Ok(Some(Self::HighSpeedCraftHazardousCategoryD)),
            45..=48 => Ok(Some(Self::HighSpeedCraftReserved)),
            49 => Ok(Some(Self::HighSpeedCraftNoAdditionalInformation)),
            50 => Ok(Some(Self::PilotVessel)),
            51 => Ok(Some(Self::SearchAndRescueVessel)),
            52 => Ok(Some(Self::Tug)),
            53 => Ok(Some(Self::PortTender)),
            54 => Ok(Some(Self::AntiPollutionEquipment)),
            55 => Ok(Some(Self::LawEnforcement)),
            56..=57 => Ok(Some(Self::SpareLocalVessel)),
            58 => Ok(Some(Self::MedicalTransport)),
            59 => Ok(Some(Self::NoncombatantShip)),
            60 => Ok(Some(Self::Passenger)),
            61 => Ok(Some(Self::PassengerHazardousCategoryA)),
            62 => Ok(Some(Self::PassengerHazardousCategoryB)),
            63 => Ok(Some(Self::PassengerHazardousCategoryC)),
            64 => Ok(Some(Self::PassengerHazardousCategoryD)),
            65..=68 => Ok(Some(Self::PassengerReserved)),
            69 => Ok(Some(Self::PassengerNoAdditionalInformation)),
            70 => Ok(Some(Self::Cargo)),
            71 => Ok(Some(Self::CargoHazardousCategoryA)),
            72 => Ok(Some(Self::CargoHazardousCategoryB)),
            73 => Ok(Some(Self::CargoHazardousCategoryC)),
            74 => Ok(Some(Self::CargoHazardousCategoryD)),
            75..=78 => Ok(Some(Self::CargoReserved)),
            79 => Ok(Some(Self::CargoNoAdditionalInformation)),
            80 => Ok(Some(Self::Tanker)),
            81 => Ok(Some(Self::TankerHazardousCategoryA)),
            82 => Ok(Some(Self::TankerHazardousCategoryB)),
            83 => Ok(Some(Self::TankerHazardousCategoryC)),
            84 => Ok(Some(Self::TankerHazardousCategoryD)),
            85..=88 => Ok(Some(Self::TankerReserved)),
            89 => Ok(Some(Self::TankerNoAdditionalInformation)),
            90 => Ok(Some(Self::Other)),
            91 => Ok(Some(Self::OtherHazardousCategoryA)),
            92 => Ok(Some(Self::OtherHazardousCategoryB)),
            93 => Ok(Some(Self::OtherHazardousCategoryC)),
            94 => Ok(Some(Self::OtherHazardousCategoryD)),
            95..=98 => Ok(Some(Self::OtherReserved)),
            99 => Ok(Some(Self::OtherNoAdditionalInformation)),
            x if x > 99 => Ok(None),
            _ => Err(format!("Unknown ship type: {}", data).into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Dte {
    Ready,
    NotReady,
}

impl Default for Dte {
    fn default() -> Self {
        Dte::NotReady
    }
}

impl From<u8> for Dte {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Ready,
            1 => Self::NotReady,
            _ => panic!("DTE should be a single bit"),
        }
    }
}
