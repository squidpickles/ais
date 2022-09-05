//! ExtendedClass B Position Report (type 19)
use super::navigation::*;
use super::parsers::*;
use super::types::AssignedMode;
use super::types::Dte;
use super::types::EpfdType;
use super::AisMessageType;
use crate::errors::Result;
use crate::messages::types::ShipType;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct ExtendedClassBPositionReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub speed_over_ground: Option<f32>,
    pub position_accuracy: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub course_over_ground: Option<f32>,
    pub true_heading: Option<u16>,
    pub timestamp: u8,
    pub name: String,
    pub type_of_ship_and_cargo: Option<ShipType>,
    pub dimension_to_bow: u16,
    pub dimension_to_stern: u16,
    pub dimension_to_port: u16,
    pub dimension_to_starboard: u16,
    pub epfd_type: Option<EpfdType>,
    pub raim: bool,
    pub dte: Dte,
    pub assigned_mode: AssignedMode,
}

impl<'a> AisMessageType<'a> for ExtendedClassBPositionReport {
    fn name(&self) -> &'static str {
        "Extended Class B Position Report"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], ExtendedClassBPositionReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _regional_reserved) = take_bits::<_, u8, _, _>(8u8)(data)?;
        let (data, speed_over_ground) = map(take_bits(10u16), parse_speed_over_ground)(data)?;
        let (data, position_accuracy) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, course_over_ground) = map(take_bits(12u16), parse_cog)(data)?;
        let (data, true_heading) = map(take_bits(9u16), parse_heading)(data)?;
        let (data, timestamp) = take_bits(6u8)(data)?;
        let (data, _regional_reserved) = take_bits::<_, u8, _, _>(4u8)(data)?;
        let (data, name) = parse_6bit_ascii(data, 120)?;
        let (data, type_of_ship_and_cargo) = map(take_bits(8u8), ShipType::parse)(data)?;
        let (data, dimension_to_bow) = take_bits(9u16)(data)?;
        let (data, dimension_to_stern) = take_bits(9u16)(data)?;
        let (data, dimension_to_port) = take_bits(6u16)(data)?;
        let (data, dimension_to_starboard) = take_bits(6u16)(data)?;
        let (data, epfd_type) = map(take_bits(4u8), EpfdType::parse)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, dte) = map(take_bits::<_, u8, _, _>(1u8), Into::into)(data)?;
        let (data, assigned_mode) = map(take_bits(1u8), AssignedMode::parse)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(4u8)(data)?;
        Ok((
            data,
            ExtendedClassBPositionReport {
                message_type,
                repeat_indicator,
                mmsi,
                speed_over_ground,
                position_accuracy,
                longitude,
                latitude,
                course_over_ground,
                true_heading,
                timestamp,
                name,
                type_of_ship_and_cargo,
                dimension_to_bow,
                dimension_to_stern,
                dimension_to_port,
                dimension_to_starboard,
                epfd_type,
                raim,
                dte,
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
    fn test_position() {
        let bytestream = b"C6:ijoP00:9NNF4TEspILDN0Vc0jNc1WWV0000000000S2<6R20P";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = ExtendedClassBPositionReport::parse(&bitstream).unwrap();
        assert_eq!(report.message_type, 19);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 413954782);
        assert_eq!(report.speed_over_ground, Some(0.0));
        assert_eq!(report.position_accuracy, Accuracy::Dgps);
        f32_equal_naive(report.longitude.unwrap(), 120.12851);
        f32_equal_naive(report.latitude.unwrap(), 31.931623);
        assert_eq!(report.course_over_ground, Some(40.7));
        assert_eq!(report.true_heading, Some(40));
        assert_eq!(report.timestamp, 60);
        assert_eq!(report.name, "SU YOU 333");
        assert_eq!(report.type_of_ship_and_cargo, Some(ShipType::Cargo));
        assert_eq!(report.dimension_to_bow, 35);
        assert_eq!(report.dimension_to_stern, 13);
        assert_eq!(report.dimension_to_port, 4);
        assert_eq!(report.dimension_to_starboard, 4);
        assert_eq!(report.epfd_type, None);
        assert!(!report.raim);
        assert_eq!(report.dte, Dte::NotReady);
        assert_eq!(report.assigned_mode, AssignedMode::Autonomous);
    }
}
