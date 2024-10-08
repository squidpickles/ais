//! Addressed Safety-Related Message (type 12)
use super::parsers::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct AddressedSafetyRelatedMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub seqno: u8,
    pub dest_mmsi: u32,
    pub retransmit: bool,
    pub text: AsciiString,
}

impl<'a> AisMessageType<'a> for AddressedSafetyRelatedMessage {
    fn name(&self) -> &'static str {
        "Addressed Safety-Related Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], AddressedSafetyRelatedMessage> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, seqno) = take_bits(2u8)(data)?;
        let (data, dest_mmsi) = take_bits(30u32)(data)?;
        let (data, retransmit) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(1u8)(data)?;

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
            AddressedSafetyRelatedMessage {
                message_type,
                repeat_indicator,
                mmsi,
                seqno,
                dest_mmsi,
                retransmit,
                text,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type12_example_a() {
        let bytestream = b"<5?SIj1;GbD07??4";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = AddressedSafetyRelatedMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 12);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 351853000);
        assert_eq!(report.seqno, 0);
        assert_eq!(report.dest_mmsi, 316123456);
        assert!(!report.retransmit);
        assert_eq!(report.text, "GOOD");
    }

    #[test]
    fn test_type12_example_b() {
        let bytestream = b"<42Lati0W:Ov=C7P6B?=Pjoihhjhqq0";
        let bitstream = crate::messages::unarmor(bytestream, 2).unwrap();
        let report = AddressedSafetyRelatedMessage::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 12);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 271002099);
        assert_eq!(report.seqno, 0);
        assert_eq!(report.dest_mmsi, 271002111);
        assert!(report.retransmit);
        assert_eq!(report.text, "MSG FROM 271002099");
    }
}
