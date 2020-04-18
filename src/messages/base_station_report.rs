use super::common::*;
use super::navigation::*;
use super::radio_status::{parse_radio, RadioStatus};
use super::{signed_i32, u8_to_bool, AisMessageType, BitStream};
use crate::errors::*;
use nom::*;

#[derive(Debug)]
pub struct BaseStationReport {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
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

    fn parse(data: BitStream) -> Result<Self> {
        match base_parser(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
            IResult::Incomplete(_) => Err("incomplete AIS sentence".into()),
        }
    }
}

named!(year_parser<(&[u8], usize), Option<u16>>, map_res!(take_bits!(u16, 14), |year| match year {
    0 => Ok(None),
    1..=9999 => Ok(Some(year)),
    _ => Err("Invalid year"),
}));

named!(month_parser<(&[u8], usize), Option<u8>>, map_res!(take_bits!(u8, 4), |month| match month {
    0 => Ok(None),
    1..=12 => Ok(Some(month)),
    _ => Err("Invalid month"),
}));

named!(day_parser<(&[u8], usize), Option<u8>>, map_res!(take_bits!(u8, 5), |day| match day {
    0 => Ok(None),
    1..=31 => Ok(Some(day)),
    _ => Err("Invalid day"),
}));

named!(hour_parser<(&[u8], usize), Option<u8>>, map_res!(take_bits!(u8, 5), |hour| match hour {
    0..=23 => Ok(Some(hour)),
    24 => Ok(None),
    _ => Err("Invalid hour"),
}));

named!(minsec_parser<(&[u8], usize), Option<u8>>, map_res!(take_bits!(u8, 6), |min_sec| match min_sec {
    0..=59 => Ok(Some(min_sec)),
    60 => Ok(None),
    _ => Err("Invalid minute/second"),
}));

named!(
    base_parser<BaseStationReport>,
    bits!(do_parse!(
        msg_type: take_bits!(u8, 6)
            >> repeat: take_bits!(u8, 2)
            >> mmsi: take_bits!(u32, 30)
            >> year: call!(year_parser)
            >> month: call!(month_parser)
            >> day: call!(day_parser)
            >> hour: call!(hour_parser)
            >> minute: call!(minsec_parser)
            >> second: call!(minsec_parser)
            >> accuracy: map_res!(take_bits!(u8, 1), Accuracy::parse)
            >> lon: map_res!(apply!(signed_i32, 28), parse_longitude)
            >> lat: map_res!(apply!(signed_i32, 27), parse_latitude)
            >> epfd: map_res!(take_bits!(u8, 4), EpfdType::parse)
            >> spare: take_bits!(u8, 10)
            >> raim: map_res!(take_bits!(u8, 1), u8_to_bool)
            >> radio: apply!(parse_radio, msg_type)
            >> (BaseStationReport {
                message_type: msg_type,
                repeat_indicator: repeat,
                mmsi: mmsi,
                year: year,
                month: month,
                day: day,
                hour: hour,
                minute: minute,
                second: second,
                fix_quality: accuracy,
                longitude: lon,
                latitude: lat,
                epfd_type: epfd,
                raim: raim,
                radio_status: radio,
            })
    ))
);

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
        let base = BaseStationReport::parse(&bitstream).unwrap();
        assert_eq!(base.message_type, 4);
        assert_eq!(base.repeat_indicator, 0);
        assert_eq!(base.mmsi, 003669145);
        assert_eq!(base.year, Some(2017));
        assert_eq!(base.month, Some(12));
        assert_eq!(base.day, Some(27));
        assert_eq!(base.hour, Some(17));
        assert_eq!(base.minute, Some(15));
        assert_eq!(base.second, Some(11));
        assert_eq!(base.fix_quality, Accuracy::DGPS);
        f32_equal_naive(base.longitude.unwrap(), -122.464775);
        f32_equal_naive(base.latitude.unwrap(), 37.794308);
        assert_eq!(base.epfd_type, None);
        assert_eq!(base.raim, true);
        if let RadioStatus::Sotdma(radio_status) = base.radio_status {
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
        let base = BaseStationReport::parse(&bitstream).unwrap();
        assert_eq!(base.message_type, 4);
        assert_eq!(base.repeat_indicator, 0);
        assert_eq!(base.mmsi, 3669702);
        assert_eq!(base.year, Some(2007));
        assert_eq!(base.month, Some(5));
        assert_eq!(base.day, Some(14));
        assert_eq!(base.hour, Some(19));
        assert_eq!(base.minute, Some(57));
        assert_eq!(base.second, Some(39));
        assert_eq!(base.fix_quality, Accuracy::DGPS);
        assert_eq!(base.longitude, Some(-76.35236));
        assert_eq!(base.latitude, Some(36.883766));
        assert_eq!(base.epfd_type, Some(EpfdType::Surveyed));
        assert_eq!(base.raim, false);
        if let RadioStatus::Sotdma(radio_status) = base.radio_status {
            assert_eq!(radio_status.sync_state, SyncState::UtcDirect);
            assert_eq!(radio_status.slot_timeout, 4);
            assert_eq!(radio_status.sub_message, SubMessage::SlotNumber(1503));
        } else {
            panic!("Expected SOTDMA message");
        }
    }
}
