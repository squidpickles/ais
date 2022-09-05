//! Common data types

/// Electronic Position Fixing Device type. This is the
/// type of device used for determining the object's
/// position.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EpfdType {
    Gps,
    Glonass,
    CombinedGpsAndGlonass,
    LoranC,
    Chayka,
    IntegratedNavigationSystem,
    Surveyed,
    Galileo,
    Unknown(u8),
}

impl EpfdType {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => None,
            1 => Some(Self::Gps),
            2 => Some(Self::Glonass),
            3 => Some(Self::CombinedGpsAndGlonass),
            4 => Some(Self::LoranC),
            5 => Some(Self::Chayka),
            6 => Some(Self::IntegratedNavigationSystem),
            7 => Some(Self::Surveyed),
            8 => Some(Self::Galileo),
            15 => None,
            _ => Some(Self::Unknown(data)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
    Unknown(u8),
}

impl ShipType {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => None,
            1..=19 => Some(Self::Reserved),
            20 => Some(Self::WingInGround),
            21 => Some(Self::WingInGroundHazardousCategoryA),
            22 => Some(Self::WingInGroundHazardousCategoryB),
            23 => Some(Self::WingInGroundHazardousCategoryC),
            24 => Some(Self::WingInGroundHazardousCategoryD),
            25..=29 => Some(Self::WingInGroundReserved),
            30 => Some(Self::Fishing),
            31 => Some(Self::Towing),
            32 => Some(Self::TowingLarge),
            33 => Some(Self::Dredging),
            34 => Some(Self::DivingOps),
            35 => Some(Self::MilitaryOps),
            36 => Some(Self::Sailing),
            37 => Some(Self::PleasureCraft),
            38..=39 => Some(Self::Reserved),
            40 => Some(Self::HighSpeedCraft),
            41 => Some(Self::HighSpeedCraftHazardousCategoryA),
            42 => Some(Self::HighSpeedCraftHazardousCategoryB),
            43 => Some(Self::HighSpeedCraftHazardousCategoryC),
            44 => Some(Self::HighSpeedCraftHazardousCategoryD),
            45..=48 => Some(Self::HighSpeedCraftReserved),
            49 => Some(Self::HighSpeedCraftNoAdditionalInformation),
            50 => Some(Self::PilotVessel),
            51 => Some(Self::SearchAndRescueVessel),
            52 => Some(Self::Tug),
            53 => Some(Self::PortTender),
            54 => Some(Self::AntiPollutionEquipment),
            55 => Some(Self::LawEnforcement),
            56..=57 => Some(Self::SpareLocalVessel),
            58 => Some(Self::MedicalTransport),
            59 => Some(Self::NoncombatantShip),
            60 => Some(Self::Passenger),
            61 => Some(Self::PassengerHazardousCategoryA),
            62 => Some(Self::PassengerHazardousCategoryB),
            63 => Some(Self::PassengerHazardousCategoryC),
            64 => Some(Self::PassengerHazardousCategoryD),
            65..=68 => Some(Self::PassengerReserved),
            69 => Some(Self::PassengerNoAdditionalInformation),
            70 => Some(Self::Cargo),
            71 => Some(Self::CargoHazardousCategoryA),
            72 => Some(Self::CargoHazardousCategoryB),
            73 => Some(Self::CargoHazardousCategoryC),
            74 => Some(Self::CargoHazardousCategoryD),
            75..=78 => Some(Self::CargoReserved),
            79 => Some(Self::CargoNoAdditionalInformation),
            80 => Some(Self::Tanker),
            81 => Some(Self::TankerHazardousCategoryA),
            82 => Some(Self::TankerHazardousCategoryB),
            83 => Some(Self::TankerHazardousCategoryC),
            84 => Some(Self::TankerHazardousCategoryD),
            85..=88 => Some(Self::TankerReserved),
            89 => Some(Self::TankerNoAdditionalInformation),
            90 => Some(Self::Other),
            91 => Some(Self::OtherHazardousCategoryA),
            92 => Some(Self::OtherHazardousCategoryB),
            93 => Some(Self::OtherHazardousCategoryC),
            94 => Some(Self::OtherHazardousCategoryD),
            95..=98 => Some(Self::OtherReserved),
            99 => Some(Self::OtherNoAdditionalInformation),
            x if x > 99 => None,
            _ => Some(Self::Unknown(data)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AssignedMode {
    Autonomous,
    Assigned,
}

impl AssignedMode {
    pub fn parse(val: u8) -> Self {
        match val {
            0 => Self::Autonomous,
            1 => Self::Assigned,
            _ => unreachable!(),
        }
    }
}
