//! Assignment Mode Command (type 16)

use super::parsers::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::IResult;

#[derive(Debug, PartialEq, Eq)]
pub struct AssignmentModeCommand {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub mmsi1: u32,
    pub offset1: u16,
    pub increment1: u16,
    pub mmsi2: Option<u32>,
    pub offset2: Option<u16>,
    pub increment2: Option<u16>,
}

impl<'a> AisMessageType<'a> for AssignmentModeCommand {
    fn name(&self) -> &'static str {
        "Assignment Mode Command"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], AssignmentModeCommand> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;

        let (data, mmsi1) = take_bits(30u32)(data)?;
        let (data, offset1) = take_bits(12u16)(data)?;
        let (data, increment1) = take_bits(10u16)(data)?;

        // Check for remaining bits, if there are enough bits for the second station
        let remaining_bits = remaining_bits(data);

        if remaining_bits >= 52 {
            let (data, mmsi2) = take_bits(30u32)(data)?;
            let (data, offset2) = take_bits(12u16)(data)?;
            let (data, increment2) = take_bits(10u16)(data)?;

            Ok((
                data,
                AssignmentModeCommand {
                    message_type,
                    repeat_indicator,
                    mmsi,
                    mmsi1,
                    offset1,
                    increment1,
                    mmsi2: Some(mmsi2),
                    offset2: Some(offset2),
                    increment2: Some(increment2),
                },
            ))
        } else {
            // Only one station assignment
            Ok((
                data,
                AssignmentModeCommand {
                    message_type,
                    repeat_indicator,
                    mmsi,
                    mmsi1,
                    offset1,
                    increment1,
                    mmsi2: None,
                    offset2: None,
                    increment2: None,
                },
            ))
        }
    })(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type16_example_1_single_station() {
        let bytestream = b"@01uEO@mMk7P<P00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = AssignmentModeCommand::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 16);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2053501);
        assert_eq!(report.mmsi1, 224251000);
        assert_eq!(report.offset1, 200);
        assert_eq!(report.increment1, 0);
        assert_eq!(report.mmsi2, None);
        assert_eq!(report.offset2, None);
        assert_eq!(report.increment2, None);
    }

    #[test]
    fn test_type16_example_2_single_station() {
        let bytestream = b"@01uEO@hsqJ0<P00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = AssignmentModeCommand::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 16);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 2053501);
        assert_eq!(report.mmsi1, 205252000);
        assert_eq!(report.offset1, 200);
        assert_eq!(report.increment1, 0);
        assert_eq!(report.mmsi2, None);
        assert_eq!(report.offset2, None);
        assert_eq!(report.increment2, None);
    }

    #[test]
    fn test_type16_example_with_two_stations() {
        let bytestream = b"@6STUk004lQ206bCKNOBAb6SJ@5s";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let report = AssignmentModeCommand::parse(bitstream.as_ref()).unwrap();

        assert_eq!(report.message_type, 16);
        assert_eq!(report.repeat_indicator, 0);
        assert_eq!(report.mmsi, 439952844);
        assert_eq!(report.mmsi1, 315920);
        assert_eq!(report.offset1, 2049);
        assert_eq!(report.increment1, 681);
        assert_eq!(report.mmsi2, Some(230137673));
        assert_eq!(report.offset2, Some(424));
        assert_eq!(report.increment2, Some(419));
    }
}
