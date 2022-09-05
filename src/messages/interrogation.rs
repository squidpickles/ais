//! Interrogation (type 15)
use super::parsers::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub message_type: u8,
    pub slot_offset: Option<u16>,
}

impl Message {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, slot_offset) = if remaining_bits(data) >= 12 {
            let (data, slot_offset) = take_bits(12u16)(data)?;
            if slot_offset == 0 {
                (data, None)
            } else {
                (data, Some(slot_offset))
            }
        } else {
            (data, None)
        };
        Ok((
            data,
            Self {
                message_type,
                slot_offset,
            },
        ))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Station {
    pub mmsi: u32,
    pub messages: Vec<Message>,
}

impl Station {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, mmsi) = take_bits(30u32)(data)?;
        let mut messages = Vec::new();
        let (data, message) = Message::parse(data)?;
        messages.push(message);
        let data = if remaining_bits(data) >= 8 {
            let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
            let (data, message) = Message::parse(data)?;
            if message.message_type != 0 || message.slot_offset.is_some() {
                messages.push(message);
            }
            data
        } else {
            data
        };
        Ok((data, Self { mmsi, messages }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Interrogation {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub stations: Vec<Station>,
}

impl<'a> AisMessageType<'a> for Interrogation {
    fn name(&self) -> &'static str {
        "Interrogation"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_message(data)?;
        Ok(report)
    }
}

fn parse_message(data: &[u8]) -> IResult<&[u8], Interrogation> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
        let mut stations = Vec::new();
        let (data, station) = Station::parse(data)?;
        stations.push(station);
        let remaining = remaining_bits(data);
        let data = if remaining >= 30 {
            let (data, station) = Station::parse(data)?;
            stations.push(station);
            take_bits::<_, u8, _, _>(2u8)(data)?.0
        } else {
            (<&[u8]>::default(), 0)
        };

        Ok((
            data,
            Interrogation {
                message_type,
                repeat_indicator,
                mmsi,
                stations,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;

    #[test]
    fn test_type15_short() {
        let bytestream = b"?03Owo@nwsI0D00";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let message = Interrogation::parse(&bitstream).unwrap();
        assert_eq!(message.message_type, 15);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 3669981);
        assert_eq!(message.stations.len(), 1);
        let station = &message.stations[0];
        assert_eq!(station.mmsi, 230682000);
        assert_eq!(station.messages.len(), 1);
        let submessage = &station.messages[0];
        assert_eq!(submessage.message_type, 5);
        assert!(submessage.slot_offset.is_none());
    }

    #[test]
    fn test_type15_busy() {
        let bytestream = b"?>eq`dAh3`TQP00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = Interrogation::parse(&bitstream).unwrap();
        assert_eq!(message.message_type, 15);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 987654321);
        assert_eq!(message.stations.len(), 1);
        let station = &message.stations[0];
        assert_eq!(station.mmsi, 470000200);
        assert_eq!(station.messages.len(), 1);
        let submessage = &station.messages[0];
        assert_eq!(submessage.message_type, 24);
        assert!(submessage.slot_offset.is_none());
    }

    #[test]
    fn test_type15_longer() {
        let bytestream = b"?04759iVhc2lD003000";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let message = Interrogation::parse(&bitstream).unwrap();
        assert_eq!(message.message_type, 15);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 4310311);
        assert_eq!(message.stations.len(), 1);
        let station = &message.stations[0];
        assert_eq!(station.mmsi, 431008813);
        assert_eq!(station.messages.len(), 2);
        let submessage = &station.messages[0];
        assert_eq!(submessage.message_type, 5);
        assert!(submessage.slot_offset.is_none());
        let submessage = &station.messages[1];
        assert_eq!(submessage.message_type, 3);
        assert!(submessage.slot_offset.is_none());
    }
}
