//! UTC/Date Inquiry (type 10)
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct UtcDateInquiry {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub dest_mmsi: u32,
}

impl<'a> AisMessageType<'a> for UtcDateInquiry {
    fn name(&self) -> &'static str {
        "UTC/Date Inquiry"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], UtcDateInquiry> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare1) = take_bits::<_, u8, _, _>(2u8)(data)?;
        let (data, dest_mmsi) = take_bits(30u32)(data)?;
        let (data, _spare2) = take_bits::<_, u8, _, _>(2u8)(data)?;

        Ok((
            data,
            UtcDateInquiry {
                message_type,
                repeat_indicator,
                mmsi,
                dest_mmsi,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type10_example_a() {
        let bytestream = b":5MlU41GMK6@";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = UtcDateInquiry::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 10);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 366814480);
        assert_eq!(report.dest_mmsi, 366832740);
    }

    #[test]
    fn test_type10_example_b() {
        let bytestream = b":6TMCD1GOS60";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = UtcDateInquiry::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 10);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 440882000);
        assert_eq!(report.dest_mmsi, 366972000);
    }
}
