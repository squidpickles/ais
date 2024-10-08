//! Safety-Related Broadcast Message (type 14)

use super::parsers::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct SafetyRelatedBroadcastMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub text: AsciiString,
}

impl<'a> AisMessageType<'a> for SafetyRelatedBroadcastMessage {
    fn name(&self) -> &'static str {
        "Safety-Related Broadcast Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], SafetyRelatedBroadcastMessage> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;

        // Ensure there are enough bits remaining for text
        let remaining_bits = remaining_bits(data);
        if remaining_bits < 6 {
            return Err(nom::Err::Error(nom::error::Error::new(
                data,
                nom::error::ErrorKind::Eof,
            )));
        }

        let (data, text) = parse_6bit_ascii(data, remaining_bits)?;

        Ok((
            data,
            SafetyRelatedBroadcastMessage {
                message_type,
                repeat_indicator,
                mmsi,
                text,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type14_example() {
        let bytestream = b">5?Per18=HB1U:1@E=B0m<L";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let report = SafetyRelatedBroadcastMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 14);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 351809000);
        assert_eq!(report.text, "RCVD YR TEST MSG");
    }
}
