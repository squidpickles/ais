//! Aid to Navigation Report (type 21)
use super::navigation::*;
use super::parsers::*;
use super::types::*;
use super::{signed_i32, u8_to_bool, AisMessageType};
use crate::errors::*;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map_res;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum NavaidType {
    ReferencePoint,
    Racon,
    FixedStructureOffShore,
    Spare,
    LightWithoutSectors,
    LightWithSectors,
    LeadingLightFront,
    LeadingLightRear,
    BeaconCardinalN,
    BeaconCardinalE,
    BeaconCardinalS,
    BeaconCardinalW,
    BeaconPortHand,
    BeaconStarboardHand,
    BeaconPreferredChannelPortHand,
    BeaconPreferredChannelStarboardHand,
    BeaconIsolatedDanger,
    BeaconSafeWater,
    BeaconSpecialMark,
    CardinalMarkN,
    CardinalMarkE,
    CardinalMarkS,
    CardinalMarkW,
    PortHandMark,
    StarboardHandMark,
    PreferredChannelPortHand,
    PreferredChannelStarboardHand,
    IsolatedDanger,
    SafeWater,
    SpecialMark,
    LightVesselOrLanbyOrRigs,
}

impl NavaidType {
    pub fn parse(data: u8) -> Result<Option<Self>> {
        match data {
            0 => Ok(None),
            1 => Ok(Some(NavaidType::ReferencePoint)),
            2 => Ok(Some(NavaidType::Racon)),
            3 => Ok(Some(NavaidType::FixedStructureOffShore)),
            4 => Ok(Some(NavaidType::Spare)),
            5 => Ok(Some(NavaidType::LightWithoutSectors)),
            6 => Ok(Some(NavaidType::LightWithSectors)),
            7 => Ok(Some(NavaidType::LeadingLightFront)),
            8 => Ok(Some(NavaidType::LeadingLightRear)),
            9 => Ok(Some(NavaidType::BeaconCardinalN)),
            10 => Ok(Some(NavaidType::BeaconCardinalE)),
            11 => Ok(Some(NavaidType::BeaconCardinalS)),
            12 => Ok(Some(NavaidType::BeaconCardinalW)),
            13 => Ok(Some(NavaidType::BeaconPortHand)),
            14 => Ok(Some(NavaidType::BeaconStarboardHand)),
            15 => Ok(Some(NavaidType::BeaconPreferredChannelPortHand)),
            16 => Ok(Some(NavaidType::BeaconPreferredChannelStarboardHand)),
            17 => Ok(Some(NavaidType::BeaconIsolatedDanger)),
            18 => Ok(Some(NavaidType::BeaconSafeWater)),
            19 => Ok(Some(NavaidType::BeaconSpecialMark)),
            20 => Ok(Some(NavaidType::CardinalMarkN)),
            21 => Ok(Some(NavaidType::CardinalMarkE)),
            22 => Ok(Some(NavaidType::CardinalMarkS)),
            23 => Ok(Some(NavaidType::CardinalMarkW)),
            24 => Ok(Some(NavaidType::PortHandMark)),
            25 => Ok(Some(NavaidType::StarboardHandMark)),
            26 => Ok(Some(NavaidType::PreferredChannelPortHand)),
            27 => Ok(Some(NavaidType::PreferredChannelStarboardHand)),
            28 => Ok(Some(NavaidType::IsolatedDanger)),
            29 => Ok(Some(NavaidType::SafeWater)),
            30 => Ok(Some(NavaidType::SpecialMark)),
            31 => Ok(Some(NavaidType::LightVesselOrLanbyOrRigs)),
            _ => Err(format!("Unknown navaid type: {}", data).into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AidToNavigationReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub aid_type: Option<NavaidType>,
    pub name: String,
    pub accuracy: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub dimension_to_bow: u16,
    pub dimension_to_stern: u16,
    pub dimension_to_port: u16,
    pub dimension_to_starboard: u16,
    pub epfd_type: Option<EpfdType>,
    pub utc_second: u8,
    pub off_position: bool,
    pub regional_reserved: u8,
    pub raim: bool,
    pub virtual_aid: bool,
    pub assigned_mode: bool,
}

impl<'a> AisMessageType<'a> for AidToNavigationReport {
    fn name(&self) -> &'static str {
        "Aid to Navigation Report"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_message(data)?;
        Ok(report)
    }
}

fn parse_message(data: &[u8]) -> IResult<&[u8], AidToNavigationReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, repeat_indicator) = take_bits::<_, _, _, (_, _)>(2u8)(data)?;
        let (data, mmsi) = take_bits::<_, _, _, (_, _)>(30u32)(data)?;
        let (data, aid_type) = map_res(take_bits::<_, _, _, (_, _)>(5u8), NavaidType::parse)(data)?;
        let (data, name) = parse_6bit_ascii(data, 120)?;
        let (data, accuracy) = map_res(take_bits::<_, _, _, (_, _)>(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map_res(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map_res(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, dimension_to_bow) = take_bits::<_, _, _, (_, _)>(9u16)(data)?;
        let (data, dimension_to_stern) = take_bits::<_, _, _, (_, _)>(9u16)(data)?;
        let (data, dimension_to_port) = take_bits::<_, _, _, (_, _)>(6u16)(data)?;
        let (data, dimension_to_starboard) = take_bits::<_, _, _, (_, _)>(6u16)(data)?;
        let (data, epfd_type) = map_res(take_bits::<_, _, _, (_, _)>(4u8), EpfdType::parse)(data)?;
        let (data, utc_second) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, off_position) = map_res(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, regional_reserved) = take_bits::<_, _, _, (_, _)>(8u8)(data)?;
        let (data, raim) = map_res(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, virtual_aid) = map_res(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, assigned_mode) = map_res(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, (_, _)>(1u8)(data)?;
        Ok((
            data,
            AidToNavigationReport {
                message_type,
                repeat_indicator,
                mmsi,
                aid_type,
                name,
                accuracy,
                longitude,
                latitude,
                dimension_to_bow,
                dimension_to_stern,
                dimension_to_port,
                dimension_to_starboard,
                epfd_type,
                utc_second,
                off_position,
                regional_reserved,
                raim,
                virtual_aid,
                assigned_mode,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_type21_not_extended() {
        let bytestream = b"E>kb9II9S@0`8@:9ah;0TahIW@@;Uafb:r5Ih00003vP100";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = AidToNavigationReport::parse(&bitstream).unwrap();
        assert_eq!(message.message_type, 21);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 993692005);
        assert_eq!(message.name, "SF APP TSS VAIS 3N");
        assert_eq!(message.accuracy, Accuracy::Unaugmented);
        f32_equal_naive(message.longitude.unwrap(), -123.35972);
        f32_equal_naive(message.latitude.unwrap(), 38.124718);
        assert_eq!(message.epfd_type, Some(EpfdType::Surveyed));
        assert_eq!(message.raim, false);
    }
}
