use errors::*;
use nom::IResult;
use super::BitStream;

#[derive(Debug, PartialEq)]
pub enum RadioStatus {
    Sotdma(SotdmaMessage),
    Itdma(ItdmaMessage),
}

impl RadioStatus {
    pub fn parse_sotdma(data: BitStream) -> Result<Self> {
        Ok(SotdmaMessage::parse(data)?)
    }

    pub fn parse_itdma(data: BitStream) -> Result<Self> {
        Ok(ItdmaMessage::parse(data)?)
    }
}

#[derive(Debug, PartialEq)]
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
            0 => {
                let result = try_parse!(input, Self::slot_offset);
                IResult::Done(result.0, SubMessage::SlotOffset(result.1))
            }
            1 => {
                let result = try_parse!(input, Self::utc_hour_and_minute);
                IResult::Done(
                    result.0,
                    SubMessage::UtcHourAndMinute((result.1).0, (result.1).1),
                )
            }
            2 | 4 | 6 => {
                let result = try_parse!(input, Self::subm_u16);
                IResult::Done(result.0, SubMessage::SlotNumber(result.1))
            }
            3 | 5 | 7 => {
                let result = try_parse!(input, Self::subm_u16);
                IResult::Done(result.0, SubMessage::ReceivedStations(result.1))
            }
            _ => unreachable!(),
        }
    }

    named!(utc_hour_and_minute<(&[u8], usize), (u8, u8)>, do_parse!(
        h: take_bits!(u8, 5) >>
        take_bits!(u8, 1) >>
        m: take_bits!(u8, 6) >>
        take_bits!(u8, 2) >>
        (h, m)
    ));
    named!(slot_offset<(&[u8], usize), i16>, take_bits!(i16, 14));
    named!(subm_u16<(&[u8], usize), u16>, take_bits!(u16, 14));
}

#[derive(Debug, PartialEq)]
pub struct SotdmaMessage {
    pub sync_state: SyncState,
    pub slot_timeout: u8,
    pub sub_message: SubMessage,
}

impl SotdmaMessage {
    pub fn parse(data: BitStream) -> Result<RadioStatus> {
        match Self::parse_impl(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing SOTDMA status")?,
            IResult::Incomplete(_) => Err("incomplete SOTDMA status")?,
        }
    }
    named!(parse_impl<RadioStatus>, bits!(Self::parser));

    named!(
        pub parser<(&[u8], usize), RadioStatus>,
        do_parse!(
            sync_state: map_res!(take_bits!(u8, 2), SyncState::parse) >> slot_timeout: take_bits!(u8, 3)
                >> sub_message: apply!(SubMessage::parse, slot_timeout) >> (RadioStatus::Sotdma(Self {
                sync_state: sync_state,
                slot_timeout: slot_timeout,
                sub_message: sub_message,
            }))
        )
    );
}

#[derive(Debug, PartialEq)]
pub struct ItdmaMessage {
    pub sync_state: SyncState,
    pub slot_increment: i16,
    pub num_slots: u8,
    pub keep: bool,
}

impl ItdmaMessage {
    pub fn parse(data: BitStream) -> Result<RadioStatus> {
        match Self::parse_impl(data) {
            IResult::Done(_, result) => Ok(result),
            IResult::Error(err) => Err(err).chain_err(|| "parsing ITDMA status")?,
            IResult::Incomplete(_) => Err("incomplete ITDMA status")?,
        }
    }

    named!(parse_impl<RadioStatus>, bits!(Self::parser));

    named!(
        pub parser<(&[u8], usize), RadioStatus>,
        do_parse!(
            sync_state: map_res!(take_bits!(u8, 2), SyncState::parse) >> slot_increment: take_bits!(i16, 13)
                >> num_slots: take_bits!(u8, 3) >> keep: take_bits!(u8, 1)
                >> (RadioStatus::Itdma(Self {
                    sync_state: sync_state,
                    slot_increment: slot_increment,
                    num_slots: num_slots,
                    keep: keep != 0,
                }))
        )
    );
}

pub fn parse_radio(input: (&[u8], usize), msg_type: u8) -> IResult<(&[u8], usize), RadioStatus> {
    match msg_type {
        1 | 2 | 4 => SotdmaMessage::parser(input),
        3 => ItdmaMessage::parser(input),
        _ => ::nom::IResult::Error(::nom::ErrorKind::Digit),
    }
}
