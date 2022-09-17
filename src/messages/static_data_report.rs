//! Static Data Report (type 24)
use super::parsers::*;
use super::types::*;
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct StaticDataReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub message_part: MessagePart,
}

impl<'a> AisMessageType<'a> for StaticDataReport {
    fn name(&self) -> &'static str {
        "Static Data Report"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_message(data)?;
        Ok(report)
    }
}

#[derive(Debug, PartialEq, Eq)]
/// Static Data Report messages have two different sub-message types.
/// The idea is that both get broadcast periodically.
pub enum MessagePart {
    /// Part A contains just the vessel name
    PartA {
        /// Name of the vessel in the report
        vessel_name: AsciiString,
    },
    /// Part B is further split into two parts, depending on whether
    /// the broadcasting entity is an auxiliary craft, or of the main
    /// ship
    PartB {
        ship_type: Option<ShipType>,
        vendor_id: AsciiString,
        model_serial: AsciiString,
        unit_model_code: u8,
        serial_number: u32,
        callsign: AsciiString,
        dimension_to_bow: u16,
        dimension_to_stern: u16,
        dimension_to_port: u16,
        dimension_to_starboard: u16,
    },
    Unknown(u8),
}

fn parse_message_part(data: (&[u8], usize)) -> IResult<(&[u8], usize), MessagePart> {
    let (data, part_number) = take_bits(2u8)(data)?;
    match part_number {
        0 => {
            // Part A
            let (data, vessel_name) = parse_6bit_ascii(data, 120)?;
            // Senders occasionally skip sending the spare bits, so this is optional
            let (data, _spare) =
                take_bits::<_, u8, _, _>(lib::std::cmp::min(remaining_bits(data), 7))(data)?;
            Ok((data, MessagePart::PartA { vessel_name }))
        }
        1 => {
            // Part B
            let (data, ship_type) = map(take_bits(8u8), ShipType::parse)(data)?;
            // vendor ID sometimes is a long string, and sometimes is a short string with attached model
            // and serial number. We'll parse both ways and present them
            let (data, vendor_id) = parse_6bit_ascii(data, 18)?;
            let (_, model_serial) = parse_6bit_ascii(data, 24)?;
            let (data, unit_model_code) = take_bits(4u8)(data)?;
            let (data, serial_number) = take_bits(20u32)(data)?;
            let (data, callsign) = parse_6bit_ascii(data, 42)?;
            let (data, dimension_to_bow) = take_bits(9u16)(data)?;
            let (data, dimension_to_stern) = take_bits(9u16)(data)?;
            let (data, dimension_to_port) = take_bits(6u16)(data)?;
            let (data, dimension_to_starboard) = take_bits(6u16)(data)?;
            let (data, _spare) = take_bits::<_, u8, _, _>(6u8)(data)?;
            Ok((
                data,
                MessagePart::PartB {
                    ship_type,
                    vendor_id,
                    model_serial,
                    unit_model_code,
                    serial_number,
                    callsign,
                    dimension_to_bow,
                    dimension_to_stern,
                    dimension_to_port,
                    dimension_to_starboard,
                },
            ))
        }
        2 | 3 => Ok((data, MessagePart::Unknown(part_number))),
        _ => unreachable!(),
    }
}

fn parse_message(data: &[u8]) -> IResult<&[u8], StaticDataReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, message_part) = parse_message_part(data)?;
        Ok((
            data,
            StaticDataReport {
                message_type,
                repeat_indicator,
                mmsi,
                message_part,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;

    #[test]
    fn test_part_a_message() {
        let bytestream = b"H6:lEgQL4r1<QDr0P4pN3KSKP00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = StaticDataReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.mmsi, 413996478);
        match message.message_part {
            MessagePart::PartA { vessel_name } => {
                assert_eq!(vessel_name, "WAN SHUN HANG 6868");
            }
            _ => panic!("Expected Message Part A"),
        }
    }

    #[test]
    fn test_part_b_main_vessel_message() {
        let bytestream = b"H3mr@L4NC=D62?P<7nmpl00@8220";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = StaticDataReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.mmsi, 257855600);
        match message.message_part {
            MessagePart::PartB {
                ship_type,
                vendor_id,
                model_serial,
                callsign,
                dimension_to_stern,
                ..
            } => {
                assert_eq!(ship_type, Some(ShipType::Fishing));
                assert_eq!(vendor_id, "SMT");
                assert_eq!(model_serial, "FBO");
                assert_eq!(callsign, "LG6584");
                assert_eq!(dimension_to_stern, 8);
            }
            _ => panic!("Expected Message Part B"),
        }
    }

    #[test]
    fn test_part_b_auxiliary_vessel_message() {
        let bytestream = b"H>cfmI4UFC@0DAN00000000H3110";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = StaticDataReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.mmsi, 985380196);
        match message.message_part {
            MessagePart::PartB {
                ship_type,
                vendor_id,
                serial_number,
                dimension_to_bow,
                ..
            } => {
                assert_eq!(ship_type, Some(ShipType::PleasureCraft));
                assert_eq!(vendor_id, "VSP");
                assert_eq!(serial_number, 83038);
                assert_eq!(dimension_to_bow, 3);
            }
            _ => panic!("Expected Message Part B"),
        }
    }
}
