use super::common::*;
use super::AisMessageType;
use crate::errors::*;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::{map, map_res};
use nom::IResult;

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
pub struct StaticAndVoyageRelatedData {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub ais_version: u8,
    pub imo_number: u32,
    pub callsign: String,
    pub vessel_name: String,
    pub ship_type: Option<ShipType>,
    pub dimension_to_bow: u16,
    pub dimension_to_stern: u16,
    pub dimension_to_port: u16,
    pub dimension_to_starboard: u16,
    pub epfd_type: Option<EpfdType>,
    pub eta_month_utc: Option<u8>,
    pub eta_day_utc: Option<u8>,
    pub eta_hour_utc: Option<u8>,
    pub eta_minute_utc: Option<u8>,
    pub draught: f32,
    pub destination: String,
    pub dte: Dte,
}

impl<'a> AisMessageType<'a> for StaticAndVoyageRelatedData {
    fn name(&self) -> &'static str {
        "Static and Voyage Related Data"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], StaticAndVoyageRelatedData> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, repeat_indicator) = take_bits::<_, _, _, (_, _)>(2u8)(data)?;
        let (data, mmsi) = take_bits::<_, _, _, (_, _)>(30u32)(data)?;
        let (data, ais_version) = take_bits::<_, _, _, (_, _)>(2u8)(data)?;
        let (data, imo_number) = take_bits::<_, _, _, (_, _)>(30u32)(data)?;
        let (data, callsign) = parse_6bit_ascii(data, 42)?;
        let (data, vessel_name) = parse_6bit_ascii(data, 120)?;
        let (data, ship_type) = map_res(take_bits::<_, _, _, (_, _)>(8u8), ShipType::parse)(data)?;
        let (data, dimension_to_bow) = take_bits::<_, _, _, (_, _)>(9u16)(data)?;
        let (data, dimension_to_stern) = take_bits::<_, _, _, (_, _)>(9u16)(data)?;
        let (data, dimension_to_port) = take_bits::<_, _, _, (_, _)>(6u16)(data)?;
        let (data, dimension_to_starboard) = take_bits::<_, _, _, (_, _)>(6u16)(data)?;
        let (data, epfd_type) = map_res(take_bits::<_, _, _, (_, _)>(4u8), EpfdType::parse)(data)?;
        let (data, eta_month_utc) = parse_month(data)?;
        let (data, eta_day_utc) = parse_day(data)?;
        let (data, eta_hour_utc) = parse_hour(data)?;
        let (data, eta_minute_utc) = parse_minsec(data)?;
        let (data, draught) = map(take_bits::<_, u8, _, (_, _)>(8u8), |raw_draught| {
            raw_draught as f32 / 10.0
        })(data)?;
        // Sometimes these messages are truncated.
        // First, take only up to 120 bits for destination
        let remaining_bits = data.0.len() * 8 - data.1;
        let (data, destination) = parse_6bit_ascii(data, std::cmp::min(120, remaining_bits))?;
        // Second, if there are no bits left for DTE, use the default value
        let (data, dte) = if !data.0.is_empty() {
            map(take_bits::<_, u8, _, (_, _)>(1u8), Into::into)(data)?
        } else {
            (data, Dte::default())
        };
        Ok((
            data,
            StaticAndVoyageRelatedData {
                message_type,
                repeat_indicator,
                mmsi,
                ais_version,
                imo_number,
                callsign,
                vessel_name,
                ship_type,
                dimension_to_bow,
                dimension_to_stern,
                dimension_to_port,
                dimension_to_starboard,
                epfd_type,
                eta_month_utc,
                eta_day_utc,
                eta_hour_utc,
                eta_minute_utc,
                draught,
                destination,
                dte,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::test_helpers::f32_equal_naive;

    #[test]
    fn test_type5_truncated() {
        let bytestream = b"5341U9`00000uCGCKL0u=@T4000000000000001?<@<47u;b004Sm51DQ0C@";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let base = StaticAndVoyageRelatedData::parse(&bitstream).unwrap();
        assert_eq!(base.message_type, 5);
        assert_eq!(base.repeat_indicator, 0);
        assert_eq!(base.mmsi, 205546790);
        assert_eq!(base.callsign, "OT5467");
        assert_eq!(base.ship_type, Some(ShipType::CargoNoAdditionalInformation));
        assert_eq!(base.eta_month_utc, Some(4));
        assert_eq!(base.destination, "ROTTERDAM");
        assert_eq!(base.epfd_type, None);
        assert_eq!(base.dte, Dte::Ready);
    }

    #[test]
    fn test_type5_multipart() {
        /*
        !AIVDM,2,1,2,A,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP0,0*78
        !AIVDM,2,2,2,A,00000000000,2*26
        TODO: use parser here
        */
        let bytestream = b"53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP000000000000";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let base = StaticAndVoyageRelatedData::parse(&bitstream).unwrap();
        assert_eq!(base.message_type, 5);
        assert_eq!(base.repeat_indicator, 0);
        assert_eq!(base.mmsi, 244250440);
        assert_eq!(base.callsign, "PF8793");
        assert_eq!(base.ship_type, Some(ShipType::PleasureCraft));
        f32_equal_naive(base.draught, 2.1);
        assert_eq!(base.eta_month_utc, Some(1));
        assert_eq!(base.destination, "NL LMMR");
        assert_eq!(base.epfd_type, None);
        assert_eq!(base.dte, Dte::Ready);
    }
}
