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


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ShipType {
    Reserved(u8),
    WingInGround,
    WingInGroundHazardousCategoryA,
    WingInGroundHazardousCategoryB,
    WingInGroundHazardousCategoryC,
    WingInGroundHazardousCategoryD,
    WingInGroundReserved(u8),
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
    HighSpeedCraftReserved(u8),
    HighSpeedCraftNoAdditionalInformation,
    PilotVessel,
    SearchAndRescueVessel,
    Tug,
    PortTender,
    AntiPollutionEquipment,
    LawEnforcement,
    SpareLocalVessel(u8),
    MedicalTransport,
    NoncombatantShip,
    Passenger,
    PassengerHazardousCategoryA,
    PassengerHazardousCategoryB,
    PassengerHazardousCategoryC,
    PassengerHazardousCategoryD,
    PassengerReserved(u8),
    PassengerNoAdditionalInformation,
    Cargo,
    CargoHazardousCategoryA,
    CargoHazardousCategoryB,
    CargoHazardousCategoryC,
    CargoHazardousCategoryD,
    CargoReserved(u8),
    CargoNoAdditionalInformation,
    Tanker,
    TankerHazardousCategoryA,
    TankerHazardousCategoryB,
    TankerHazardousCategoryC,
    TankerHazardousCategoryD,
    TankerReserved(u8),
    TankerNoAdditionalInformation,
    Other,
    OtherHazardousCategoryA,
    OtherHazardousCategoryB,
    OtherHazardousCategoryC,
    OtherHazardousCategoryD,
    OtherReserved(u8),
    OtherNoAdditionalInformation,
}

impl ShipType {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => None,
            1..=19 => Some(Self::Reserved(data)),
            20 => Some(Self::WingInGround),
            21 => Some(Self::WingInGroundHazardousCategoryA),
            22 => Some(Self::WingInGroundHazardousCategoryB),
            23 => Some(Self::WingInGroundHazardousCategoryC),
            24 => Some(Self::WingInGroundHazardousCategoryD),
            25..=29 => Some(Self::WingInGroundReserved(data)),
            30 => Some(Self::Fishing),
            31 => Some(Self::Towing),
            32 => Some(Self::TowingLarge),
            33 => Some(Self::Dredging),
            34 => Some(Self::DivingOps),
            35 => Some(Self::MilitaryOps),
            36 => Some(Self::Sailing),
            37 => Some(Self::PleasureCraft),
            38..=39 => Some(Self::Reserved(data)),
            40 => Some(Self::HighSpeedCraft),
            41 => Some(Self::HighSpeedCraftHazardousCategoryA),
            42 => Some(Self::HighSpeedCraftHazardousCategoryB),
            43 => Some(Self::HighSpeedCraftHazardousCategoryC),
            44 => Some(Self::HighSpeedCraftHazardousCategoryD),
            45..=48 => Some(Self::HighSpeedCraftReserved(data)),
            49 => Some(Self::HighSpeedCraftNoAdditionalInformation),
            50 => Some(Self::PilotVessel),
            51 => Some(Self::SearchAndRescueVessel),
            52 => Some(Self::Tug),
            53 => Some(Self::PortTender),
            54 => Some(Self::AntiPollutionEquipment),
            55 => Some(Self::LawEnforcement),
            56..=57 => Some(Self::SpareLocalVessel(data)),
            58 => Some(Self::MedicalTransport),
            59 => Some(Self::NoncombatantShip),
            60 => Some(Self::Passenger),
            61 => Some(Self::PassengerHazardousCategoryA),
            62 => Some(Self::PassengerHazardousCategoryB),
            63 => Some(Self::PassengerHazardousCategoryC),
            64 => Some(Self::PassengerHazardousCategoryD),
            65..=68 => Some(Self::PassengerReserved(data)),
            69 => Some(Self::PassengerNoAdditionalInformation),
            70 => Some(Self::Cargo),
            71 => Some(Self::CargoHazardousCategoryA),
            72 => Some(Self::CargoHazardousCategoryB),
            73 => Some(Self::CargoHazardousCategoryC),
            74 => Some(Self::CargoHazardousCategoryD),
            75..=78 => Some(Self::CargoReserved(data)),
            79 => Some(Self::CargoNoAdditionalInformation),
            80 => Some(Self::Tanker),
            81 => Some(Self::TankerHazardousCategoryA),
            82 => Some(Self::TankerHazardousCategoryB),
            83 => Some(Self::TankerHazardousCategoryC),
            84 => Some(Self::TankerHazardousCategoryD),
            85..=88 => Some(Self::TankerReserved(data)),
            89 => Some(Self::TankerNoAdditionalInformation),
            90 => Some(Self::Other),
            91 => Some(Self::OtherHazardousCategoryA),
            92 => Some(Self::OtherHazardousCategoryB),
            93 => Some(Self::OtherHazardousCategoryC),
            94 => Some(Self::OtherHazardousCategoryD),
            95..=98 => Some(Self::OtherReserved(data)),
            99 => Some(Self::OtherNoAdditionalInformation),
            100..=u8::MAX => None,
        }
    }
}

impl From<u8> for ShipType {
    fn from(value: u8) -> Self {
        ShipType::parse(value).unwrap()
    }
}

impl From<ShipType> for u8 {
    fn from(value: ShipType) -> u8 {
        use ShipType::*;

        match value {
            Reserved(v) => v,
            WingInGround => 20,
            WingInGroundHazardousCategoryA => 21,
            WingInGroundHazardousCategoryB => 22,
            WingInGroundHazardousCategoryC => 23,
            WingInGroundHazardousCategoryD => 24,
            WingInGroundReserved(v) => v,
            Fishing => 30,
            Towing => 31,
            TowingLarge => 32,
            Dredging => 33,
            DivingOps => 34,
            MilitaryOps => 35,
            Sailing => 36,
            PleasureCraft => 37,
            HighSpeedCraft => 40,
            HighSpeedCraftHazardousCategoryA => 41,
            HighSpeedCraftHazardousCategoryB => 42,
            HighSpeedCraftHazardousCategoryC => 43,
            HighSpeedCraftHazardousCategoryD => 44,
            HighSpeedCraftReserved(v) => v,
            HighSpeedCraftNoAdditionalInformation => 49,
            PilotVessel => 50,
            SearchAndRescueVessel => 51,
            Tug => 52,
            PortTender => 53,
            AntiPollutionEquipment => 54,
            LawEnforcement => 55,
            SpareLocalVessel(v) => v,
            MedicalTransport => 58,
            NoncombatantShip => 59,
            Passenger => 60,
            PassengerHazardousCategoryA => 61,
            PassengerHazardousCategoryB => 62,
            PassengerHazardousCategoryC => 63,
            PassengerHazardousCategoryD => 64,
            PassengerReserved(v) => v,
            PassengerNoAdditionalInformation => 69,
            Cargo => 70,
            CargoHazardousCategoryA => 71,
            CargoHazardousCategoryB => 72,
            CargoHazardousCategoryC => 73,
            CargoHazardousCategoryD => 74,
            CargoReserved(v) => v,
            CargoNoAdditionalInformation => 79,
            Tanker => 80,
            TankerHazardousCategoryA => 81,
            TankerHazardousCategoryB => 82,
            TankerHazardousCategoryC => 83,
            TankerHazardousCategoryD => 84,
            TankerReserved(v) => v,
            TankerNoAdditionalInformation => 89,
            Other => 90,
            OtherHazardousCategoryA => 91,
            OtherHazardousCategoryB => 92,
            OtherHazardousCategoryC => 93,
            OtherHazardousCategoryD => 94,
            OtherReserved(v) => v,
            OtherNoAdditionalInformation => 99,
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
