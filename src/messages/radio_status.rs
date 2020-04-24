use super::u8_to_bool;
use crate::errors::Result;
use nom::bits::complete::take as take_bits;
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum RadioStatus {
    Sotdma(SotdmaMessage),
    Itdma(ItdmaMessage),
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SyncState {
    UtcDirect,
    UtcIndirect,
    BaseStation,
    NumberOfReceivedStations,
}

impl SyncState {
    pub fn parse(data: u8) -> Result<Self> {
        match data {
            0 => Ok(SyncState::UtcDirect),
            1 => Ok(SyncState::UtcIndirect),
            2 => Ok(SyncState::BaseStation),
            3 => Ok(SyncState::NumberOfReceivedStations),
            _ => Err(format!("Unknown sync state: {}", data).into()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SubMessage {
    SlotOffset(i16),
    UtcHourAndMinute(u8, u8),
    SlotNumber(u16),
    ReceivedStations(u16),
}

impl SubMessage {
    fn parse(input: (&[u8], usize), slot_timeout: u8) -> IResult<(&[u8], usize), SubMessage> {
        match slot_timeout {
            0 => Self::slot_offset(input),
            1 => Self::utc_hour_and_minute(input),
            2 | 4 | 6 => {
                let (data, slot_number) = Self::subm_u16(input)?;
                Ok((data, SubMessage::SlotNumber(slot_number)))
            }
            3 | 5 | 7 => {
                let (data, slot_number) = Self::subm_u16(input)?;
                Ok((data, SubMessage::ReceivedStations(slot_number)))
            }
            _ => Err(nom::Err::Error((input, ErrorKind::Digit))),
        }
    }

    fn utc_hour_and_minute(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, hour) = take_bits::<_, _, _, (_, _)>(5u8)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, (_, _)>(1u8)(data)?;
        let (data, minute) = take_bits::<_, _, _, (_, _)>(6u8)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, (_, _)>(2u8)(data)?;
        Ok((data, Self::UtcHourAndMinute(hour, minute)))
    }

    fn slot_offset(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, slot_offset) = take_bits::<_, _, _, (_, _)>(14u16)(data)?;
        Ok((data, Self::SlotOffset(slot_offset)))
    }

    fn subm_u16(data: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
        take_bits::<_, _, _, (_, _)>(14u16)(data)
    }
}

#[derive(Debug, PartialEq)]
pub struct SotdmaMessage {
    pub sync_state: SyncState,
    pub slot_timeout: u8,
    pub sub_message: SubMessage,
}

impl SotdmaMessage {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), RadioStatus> {
        let (data, sync_state) =
            map_res(take_bits::<_, _, _, (_, _)>(2u8), SyncState::parse)(data)?;
        let (data, slot_timeout) = take_bits::<_, _, _, (_, _)>(3u8)(data)?;
        let (data, sub_message) = SubMessage::parse(data, slot_timeout)?;
        Ok((
            data,
            RadioStatus::Sotdma(Self {
                sync_state,
                slot_timeout,
                sub_message,
            }),
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct ItdmaMessage {
    pub sync_state: SyncState,
    pub slot_increment: i16,
    pub num_slots: u8,
    pub keep: bool,
}

impl ItdmaMessage {
    pub fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), RadioStatus> {
        let (data, sync_state) =
            map_res(take_bits::<_, _, _, (_, _)>(2u8), SyncState::parse)(data)?;
        let (data, slot_increment) = take_bits::<_, _, _, (_, _)>(13u16)(data)?;
        let (data, num_slots) = take_bits::<_, _, _, (_, _)>(3u8)(data)?;
        let (data, keep) = map_res(take_bits::<_, _, _, (_, _)>(1u8), u8_to_bool)(data)?;
        Ok((
            data,
            RadioStatus::Itdma(Self {
                sync_state,
                slot_increment,
                num_slots,
                keep,
            }),
        ))
    }
}

pub fn parse_radio(input: (&[u8], usize), msg_type: u8) -> IResult<(&[u8], usize), RadioStatus> {
    match msg_type {
        1 | 2 | 4 => SotdmaMessage::parse(input),
        3 => ItdmaMessage::parse(input),
        _ => Err(nom::Err::Error((input, ErrorKind::Digit))),
    }
}
