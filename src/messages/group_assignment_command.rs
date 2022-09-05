//! Base Station Report (type 4)
use std::time::Duration;

use super::navigation::*;
use super::parsers::*;
use super::types::*;
use super::AisMessageType;
use crate::errors::Result;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum TransmitMode {
    TxATxBRxARxB,
    TxARxARxB,
    TxBRxaRxB,
    Reserved,
}

impl From<u8> for TransmitMode {
    fn from(data: u8) -> Self {
        match data {
            0 => Self::TxATxBRxARxB,
            1 => Self::TxARxARxB,
            2 => Self::TxBRxaRxB,
            3 => Self::Reserved,
            _ => panic!("Invalid transmit mode"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum StationType {
    AllMobiles,
    Reserved,
    AllClassB,
    SarAirborneMobile,
    AidToNavigation,
    ClassBShipborneMobile,
    RegionalUseAndInlandWaterways,
}

impl From<u8> for StationType {
    fn from(data: u8) -> Self {
        match data {
            0 => Self::AllMobiles,
            1 => Self::Reserved,
            2 => Self::AllClassB,
            3 => Self::SarAirborneMobile,
            4 => Self::AidToNavigation,
            5 => Self::ClassBShipborneMobile,
            6..=9 => Self::RegionalUseAndInlandWaterways,
            10..=15 => Self::Reserved,
            _ => panic!("Invalid station type"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum StationInterval {
    AsAutonomousMode,
    Span(Duration),
    NextShorterInterval,
    NextLongerInterval,
    Reserved(u8),
}

impl From<u8> for StationInterval {
    fn from(data: u8) -> Self {
        match data {
            0 => Self::AsAutonomousMode,
            1 => Self::Span(Duration::from_secs(600)),
            2 => Self::Span(Duration::from_secs(360)),
            3 => Self::Span(Duration::from_secs(180)),
            4 => Self::Span(Duration::from_secs(60)),
            5 => Self::Span(Duration::from_secs(30)),
            6 => Self::Span(Duration::from_secs(15)),
            7 => Self::Span(Duration::from_secs(10)),
            8 => Self::Span(Duration::from_secs(5)),
            9 => Self::NextShorterInterval,
            10 => Self::NextLongerInterval,
            11..=15 => Self::Reserved(data),
            _ => panic!("Invalid station interval"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GroupAssignmentCommand {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub ne_longitude: Option<f32>,
    pub ne_latitude: Option<f32>,
    pub sw_longitude: Option<f32>,
    pub sw_latitude: Option<f32>,
    pub station_type: StationType,
    pub ship_type: Option<ShipType>,
    pub tx_rx_mode: TransmitMode,
    pub report_interval: StationInterval,
    pub quiet_time: Option<u8>,
}

impl<'a> AisMessageType<'a> for GroupAssignmentCommand {
    fn name(&self) -> &'static str {
        "Group Assignment Command"
    }

    fn parse(data: &[u8]) -> Result<Self> {
        let (_, report) = parse_base(data)?;
        Ok(report)
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], GroupAssignmentCommand> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
        let (data, ne_longitude) = map(|data| signed_i32(data, 18), parse_longitude)(data)?;
        let (data, ne_latitude) = map(|data| signed_i32(data, 17), parse_latitude)(data)?;
        let (data, sw_longitude) = map(|data| signed_i32(data, 18), parse_longitude)(data)?;
        let (data, sw_latitude) = map(|data| signed_i32(data, 17), parse_latitude)(data)?;
        let (data, station_type) = map(take_bits::<_, u8, _, _>(4u8), Into::into)(data)?;
        let (data, ship_type) = map(take_bits::<_, u8, _, _>(8u8), ShipType::parse)(data)?;
        let (data, _spare) = take_bits::<_, u32, _, _>(22u32)(data)?;
        let (data, tx_rx_mode) = map(take_bits::<_, u8, _, _>(2u8), Into::into)(data)?;
        let (data, report_interval) = map(take_bits::<_, u8, _, _>(4u8), Into::into)(data)?;
        let (data, quiet_time) = parse_quiet_time(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(6u8)(data)?;
        Ok((
            data,
            GroupAssignmentCommand {
                message_type,
                repeat_indicator,
                mmsi,
                ne_longitude,
                ne_latitude,
                sw_longitude,
                sw_latitude,
                station_type,
                ship_type,
                tx_rx_mode,
                report_interval,
                quiet_time,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::zero_prefixed_literal)]
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_type23() {
        let bytestream = b"G02OHAP8aLvg@@b1tF600000;00";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = GroupAssignmentCommand::parse(&bitstream).unwrap();
        assert_eq!(message.message_type, 23);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 2611270);
        f32_equal_naive(message.ne_longitude.unwrap(), 0.014758334);
        f32_equal_naive(message.ne_latitude.unwrap(), 0.05353667);
        f32_equal_naive(message.sw_longitude.unwrap(), 0.014213333);
        f32_equal_naive(message.sw_latitude.unwrap(), 0.053053334);
        assert_eq!(
            message.station_type,
            StationType::RegionalUseAndInlandWaterways,
        );
        assert_eq!(message.ship_type, None);
        assert_eq!(message.tx_rx_mode, TransmitMode::TxATxBRxARxB);
        assert_eq!(message.report_interval, StationInterval::NextLongerInterval);
        assert_eq!(message.quiet_time, None);
    }
}
