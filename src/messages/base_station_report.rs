//! Base Station Report (type 4)
use super::navigation::*;
use super::parsers::*;
use super::radio_status::{parse_radio, RadioStatus};
use super::types::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BaseStationReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: u8,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub fix_quality: Accuracy,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub epfd_type: Option<EpfdType>,
    pub raim: bool,
    pub radio_status: RadioStatus,
}

impl<'a> AisMessageType<'a> for BaseStationReport {
    fn name(&self) -> &'static str {
        "Base Station Report"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], BaseStationReport> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, year) = parse_year(data)?;
        let (data, month) = parse_month(data)?;
        let (data, day) = parse_day(data)?;
        let (data, hour) = parse_hour(data)?;
        let (data, minute) = parse_minsec(data)?;
        let (data, second) = parse_minsec(data)?;
        let (data, fix_quality) = map(take_bits(1u8), Accuracy::parse)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 28), parse_longitude)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 27), parse_latitude)(data)?;
        let (data, epfd_type) = map(take_bits(4u8), EpfdType::parse)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(10u8)(data)?;
        let (data, raim) = map(take_bits(1u8), u8_to_bool)(data)?;
        let (data, radio_status) = parse_radio(data, message_type)?;
        Ok((
            data,
            BaseStationReport {
                message_type,
                repeat_indicator,
                mmsi,
                year,
                month,
                day,
                hour,
                minute,
                second,
                fix_quality,
                longitude,
                latitude,
                epfd_type,
                raim,
                radio_status,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::zero_prefixed_literal)]
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::messages::radio_status::{SubMessage, SyncState};
    use crate::test_helpers::*;

    #[test]
    fn test_type4() {
        let bytestream = b"403OtVAv7=i?;o?IaHE`4Iw020S:";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = BaseStationReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 4);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 003669145);
        assert_eq!(message.year, Some(2017));
        assert_eq!(message.month, Some(12));
        assert_eq!(message.day, Some(27));
        assert_eq!(message.hour, 17);
        assert_eq!(message.minute, Some(15));
        assert_eq!(message.second, Some(11));
        assert_eq!(message.fix_quality, Accuracy::Dgps);
        f32_equal_naive(message.longitude.unwrap(), -122.464775);
        f32_equal_naive(message.latitude.unwrap(), 37.794308);
        assert_eq!(message.epfd_type, None);
        assert!(message.raim);
        if let RadioStatus::Sotdma(radio_status) = message.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 0);
            assert_eq!(radio_status.sub_message, SubMessage::SlotOffset(2250));
        } else {
            panic!("Expected SOTDMA message");
        }
    }

    #[test]
    fn test_type4_2() {
        let bytestream = b"403OviQuMGCqWrRO9>E6fE700@GO";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = BaseStationReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 4);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 3669702);
        assert_eq!(message.year, Some(2007));
        assert_eq!(message.month, Some(5));
        assert_eq!(message.day, Some(14));
        assert_eq!(message.hour, 19);
        assert_eq!(message.minute, Some(57));
        assert_eq!(message.second, Some(39));
        assert_eq!(message.fix_quality, Accuracy::Dgps);
        assert_eq!(message.longitude, Some(-76.35236));
        assert_eq!(message.latitude, Some(36.883766));
        assert_eq!(message.epfd_type, Some(EpfdType::Surveyed));
        assert!(!message.raim);
        if let RadioStatus::Sotdma(radio_status) = message.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 4);
            assert_eq!(radio_status.sub_message, SubMessage::SlotNumber(1503));
        } else {
            panic!("Expected SOTDMA message");
        }
    }

    #[test]
    fn test_type4_invalid_date() {
        let bytestream = b"4h2E:qT47wk?0<tSF0l4Q@000d;@";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = BaseStationReport::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.mmsi, 002444006);
        assert_eq!(message.year, Some(4161));
        assert_eq!(message.month, Some(15));
        assert_eq!(message.day, Some(31));
    }
}
