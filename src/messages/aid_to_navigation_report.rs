//! Aid to Navigation Report (type 21)
use super::navigation::*;
use super::parsers::*;
use super::types::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
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
    Unknown(u8),
}

impl NavaidType {
    pub fn parse(data: u8) -> Option<Self> {
        match data {
            0 => None,
            1 => Some(Self::ReferencePoint),
            2 => Some(Self::Racon),
            3 => Some(Self::FixedStructureOffShore),
            4 => Some(Self::Spare),
            5 => Some(Self::LightWithoutSectors),
            6 => Some(Self::LightWithSectors),
            7 => Some(Self::LeadingLightFront),
            8 => Some(Self::LeadingLightRear),
            9 => Some(Self::BeaconCardinalN),
            10 => Some(Self::BeaconCardinalE),
            11 => Some(Self::BeaconCardinalS),
            12 => Some(Self::BeaconCardinalW),
            13 => Some(Self::BeaconPortHand),
            14 => Some(Self::BeaconStarboardHand),
            15 => Some(Self::BeaconPreferredChannelPortHand),
            16 => Some(Self::BeaconPreferredChannelStarboardHand),
            17 => Some(Self::BeaconIsolatedDanger),
            18 => Some(Self::BeaconSafeWater),
            19 => Some(Self::BeaconSpecialMark),
            20 => Some(Self::CardinalMarkN),
            21 => Some(Self::CardinalMarkE),
            22 => Some(Self::CardinalMarkS),
            23 => Some(Self::CardinalMarkW),
            24 => Some(Self::PortHandMark),
            25 => Some(Self::StarboardHandMark),
            26 => Some(Self::PreferredChannelPortHand),
            27 => Some(Self::PreferredChannelStarboardHand),
            28 => Some(Self::IsolatedDanger),
            29 => Some(Self::SafeWater),
            30 => Some(Self::SpecialMark),
            31 => Some(Self::LightVesselOrLanbyOrRigs),
            _ => Some(Self::Unknown(data)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AidToNavigationReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub aid_type: Option<NavaidType>,
    pub name: AsciiString,
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

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_message(data)?;
        Ok(report)
    }
}

fn parse_message(data: &[u8]) -> IResult<&[u8], AidToNavigationReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, aid_type) = map(take_bits(5u8), NavaidType::parse)(data)?;
        let (data, name) = parse_6bit_ascii(data, 120)?;
        let (data, accuracy) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, dimension_to_bow) = take_bits(9u16)(data)?;
        let (data, dimension_to_stern) = take_bits(9u16)(data)?;
        let (data, dimension_to_port) = take_bits(6u16)(data)?;
        let (data, dimension_to_starboard) = take_bits(6u16)(data)?;
        let (data, epfd_type) = map(take_bits(4u8), EpfdType::parse)(data)?;
        let (data, utc_second) = take_bits(6u8)(data)?;
        let (data, off_position) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, regional_reserved) = take_bits(8u8)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, virtual_aid) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, assigned_mode) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(1u8)(data)?;
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
        let message = AidToNavigationReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 21);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 993692005);
        assert_eq!(message.name, "SF APP TSS VAIS 3N");
        assert_eq!(message.accuracy, Accuracy::Unaugmented);
        f32_equal_naive(message.longitude.unwrap(), -123.35972);
        f32_equal_naive(message.latitude.unwrap(), 38.124718);
        assert_eq!(message.epfd_type, Some(EpfdType::Surveyed));
        assert!(!message.raim);
    }
}
