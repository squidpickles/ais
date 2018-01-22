use errors::*;
use nom::IResult;
use super::{sixbit_to_ascii, AisMessage, BitStream, signed_i32, u8_to_bool};
use super::common::*;
use super::navigation::*;

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

#[derive(Debug)]
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

impl<'a> AisMessage<'a> for AidToNavigationReport {
    fn name(&self) -> &'static str {
        "Aid to Navigation Report"
    }

    fn parse(data: BitStream) -> Result<Self> {
        match base_parser(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
            IResult::Incomplete(_) => Err("incomplete AIS sentence")?,
        }
    }
}

pub fn parse_6bit_ascii(input: (&[u8], usize), size: usize) -> IResult<(&[u8], usize), String> {
    let chars = size / 6;
    if size % 6 != 0 {
        return IResult::Error(::nom::ErrorKind::LengthValue);
    }
    let mut converted: Vec<u8> = Vec::with_capacity(chars);
    let mut offset = input;
    for _ in 0..chars {
        let (new_offset, byte) = try_parse!(offset, take_bits!(u8, 6));
        offset = new_offset;
        converted.push(match sixbit_to_ascii(byte) {
            Ok(ascii) => ascii,
            Err(_) => return IResult::Error(::nom::ErrorKind::AlphaNumeric),
        });
    }
    match ::std::str::from_utf8(&converted) {
        Ok(val) => IResult::Done(offset, val.trim_right().to_string()),
        Err(_) => IResult::Error(::nom::ErrorKind::AlphaNumeric),
    }
}

named!(
    base_parser<AidToNavigationReport>,
    bits!(do_parse!(
        msg_type: take_bits!(u8, 6) >> repeat: take_bits!(u8, 2) >> mmsi: take_bits!(u32, 30)
            >> aid_type: map_res!(take_bits!(u8, 5), NavaidType::parse)
            >> name: apply!(parse_6bit_ascii, 120)
            >> accuracy: map_res!(take_bits!(u8, 1), Accuracy::parse)
            >> lon: map_res!(apply!(signed_i32, 28), parse_longitude)
            >> lat: map_res!(apply!(signed_i32, 27), parse_latitude)
            >> to_bow: take_bits!(u16, 9) >> to_stern: take_bits!(u16, 9)
            >> to_port: take_bits!(u16, 6) >> to_sb: take_bits!(u16, 6)
            >> epfd: map_res!(take_bits!(u8, 4), EpfdType::parse)
            >> stamp: take_bits!(u8, 6) >> off_pos: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> regional: take_bits!(u8, 8) >> raim: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> virtual_aid: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> assigned_mode: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> spare: take_bits!(u8, 1) >> (AidToNavigationReport {
            message_type: msg_type,
            repeat_indicator: repeat,
            mmsi: mmsi,
            aid_type: aid_type,
            name: name,
            accuracy: accuracy,
            longitude: lon,
            latitude: lat,
            dimension_to_bow: to_bow,
            dimension_to_stern: to_stern,
            dimension_to_port: to_port,
            dimension_to_starboard: to_sb,
            epfd_type: epfd,
            utc_second: stamp,
            off_position: off_pos,
            regional_reserved: regional,
            raim: raim,
            virtual_aid: virtual_aid,
            assigned_mode: assigned_mode,
        })
    ))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type21_not_extended() {
        let bytestream = b"E>kb9II9S@0`8@:9ah;0TahIW@@;Uafb:r5Ih00003vP100";
        let bitstream = ::messages::unarmor(bytestream, 0).unwrap();
        let base = AidToNavigationReport::parse(&bitstream).unwrap();
        assert_eq!(base.message_type, 21);
        assert_eq!(base.repeat_indicator, 0);
        assert_eq!(base.mmsi, 993692005);
        assert_eq!(base.name, "SF APP TSS VAIS 3N");
        assert_eq!(base.accuracy, Accuracy::Unaugmented);
        assert_eq!(base.longitude, Some(-123.35972));
        assert_eq!(base.latitude, Some(38.124718));
        assert_eq!(base.epfd_type, Some(EpfdType::Surveyed));
        assert_eq!(base.raim, false);
    }

}
