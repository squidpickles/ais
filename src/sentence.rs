//! Handlers for AIS messages at the NMEA sentence layer
use crate::errors::{ErrorKind, *};
use crate::messages::{self, AisMessage};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{anychar, digit1};
use nom::combinator::{map_res, opt, peek, verify};
use nom::number::complete::hex_u32;
use nom::sequence::terminated;
use nom::IResult;

#[derive(PartialEq, Debug)]
/// Represents the NMEA sentence type of an AIS message
pub enum AisSentenceType {
    /// Report from another ship
    AIVDM,
    /// Report from own ship
    AIVDO,
    /// Unknown AIS sentence type
    Unknown,
}

impl<'a> From<&'a [u8]> for AisSentenceType {
    fn from(typ: &'a [u8]) -> Self {
        match typ {
            b"AIVDM" => AisSentenceType::AIVDM,
            b"AIVDO" => AisSentenceType::AIVDO,
            _ => AisSentenceType::Unknown,
        }
    }
}

#[derive(PartialEq, Debug)]
/// Represents an NMEA sentence parsed as AIS, but without the internal data
/// decoded yet
pub struct AisSentence<'a> {
    pub msg_type: AisSentenceType,
    pub num_fragments: u8,
    pub fragment_number: u8,
    pub message_id: Option<u8>,
    pub channel: char,
    pub data: &'a [u8],
    pub fill_bit_count: u8,
}

impl<'a> AisSentence<'a> {
    /// Parses `line` as an NMEA sentence, checking the checksum and returning an
    /// an `AisSentence`. Note that several `AisSentence`s might be required to
    /// complete a message, if they are fragments
    pub fn parse(line: &'a [u8]) -> Result<AisSentence<'a>> {
        let (_, (data, ais_msg, checksum)) = nmea_sentence(line)?;
        Self::check_checksum(data, checksum)?;
        Ok(ais_msg)
    }

    /// Parses the message frame inside the sentence, and returns it
    pub fn message(&self) -> Result<AisMessage> {
        let raw = messages::unarmor(self.data, self.fill_bit_count as usize)?;
        Ok(messages::parse(&raw)?)
    }

    /// Verifies the checksum
    fn check_checksum(sentence: &[u8], expected_checksum: u8) -> Result<u8> {
        let received_checksum = sentence.iter().fold(0u8, |acc, &item| acc ^ item);
        if expected_checksum != received_checksum {
            Err(ErrorKind::Checksum(expected_checksum, received_checksum).into())
        } else {
            Ok(received_checksum)
        }
    }

    /// Returns whether there are more fragments to come
    pub fn has_more(&self) -> bool {
        self.fragment_number < self.num_fragments
    }
}

/// Converts bytes representing an ASCII number to a string slice
fn numeric_string(data: &[u8]) -> IResult<&[u8], &str> {
    map_res(digit1, std::str::from_utf8)(data)
}

/// Converts bytes representing an ASCII number to a u8
fn u8_digit(data: &[u8]) -> IResult<&[u8], u8> {
    map_res(numeric_string, std::str::FromStr::from_str)(data)
}

fn nmea_start(data: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((tag("!"), tag("$")))(data)
}
fn ais_type(data: &[u8]) -> IResult<&[u8], AisSentenceType> {
    let (data, sentence_type) = alt((tag("AIVDM"), tag("AIVDO")))(data)?;
    Ok((data, sentence_type.into()))
}
fn num_fragments(data: &[u8]) -> IResult<&[u8], u8> {
    u8_digit(data)
}
fn fragment_number(data: &[u8]) -> IResult<&[u8], u8> {
    u8_digit(data)
}
fn sequential_message_id(data: &[u8]) -> IResult<&[u8], Option<u8>> {
    opt(u8_digit)(data)
}
fn channel(data: &[u8]) -> IResult<&[u8], char> {
    anychar(data)
}
fn ais_data(data: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until(",")(data)
}
fn fill_bit_count(data: &[u8]) -> IResult<&[u8], u8> {
    verify(u8_digit, |val| val < &6)(data)
}
fn data_end(data: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("*")(data)
}
fn checksum(data: &[u8]) -> IResult<&[u8], u32> {
    verify(hex_u32, |val| val <= &0xff)(data)
}

fn ais_sentence(data: &[u8]) -> IResult<&[u8], AisSentence> {
    let (data, msg_type) = ais_type(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, num_fragments) = num_fragments(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, fragment_number) = fragment_number(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, message_id) = sequential_message_id(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, channel) = channel(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, ais_data) = ais_data(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, fill_bit_count) = fill_bit_count(data)?;
    Ok((
        data,
        AisSentence {
            msg_type,
            num_fragments,
            fragment_number,
            message_id,
            channel,
            data: ais_data,
            fill_bit_count,
        },
    ))
}

fn nmea_sentence(data: &[u8]) -> IResult<&[u8], (&[u8], AisSentence, u8)> {
    let (data, _) = nmea_start(data)?;
    let (data, raw) = peek(take_until("*"))(data)?;
    let (data, msg) = terminated(ais_sentence, data_end)(data)?;
    let (data, checksum) = checksum(data)?;
    Ok((data, (raw, msg, checksum as u8)))
}

#[cfg(test)]
mod tests {
    const GOOD_CHECKSUM: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A";
    const BAD_CHECKSUM: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const BAD_STRUCTURE: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0,TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const AIS_START_IDX: usize = 14;
    const AIS_END_IDX: usize = 61;
    use super::*;

    #[test]
    fn parse_valid_structure() {
        let result = ais_sentence(&GOOD_CHECKSUM[1..63]).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(
            result.1,
            AisSentence {
                msg_type: AisSentenceType::AIVDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: 'A',
                data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX],
                fill_bit_count: 0,
            }
        );
    }

    #[test]
    fn parse_invalid_structure() {
        assert!(ais_sentence(&BAD_STRUCTURE[1..64]).is_err());
    }

    #[test]
    fn parse_valid_sentence() {
        let result = nmea_sentence(GOOD_CHECKSUM).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(
            (result.1).1,
            AisSentence {
                msg_type: AisSentenceType::AIVDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: 'A',
                data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX],
                fill_bit_count: 0,
            }
        );
        assert_eq!((result.1).2, 122);
    }

    #[test]
    fn parse_using_struct_valid() {
        let result = AisSentence::parse(GOOD_CHECKSUM).unwrap();
        assert_eq!(
            result,
            AisSentence {
                msg_type: AisSentenceType::AIVDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: 'A',
                data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX],
                fill_bit_count: 0,
            }
        );
    }

    #[test]
    fn parse_valid_checksum() {
        let result = AisSentence::parse(GOOD_CHECKSUM);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_checksum() {
        let result = AisSentence::parse(BAD_CHECKSUM);
        assert!(result.is_err());
    }
}
