//! Handlers for AIS messages at the NMEA sentence layer

use crate::lib;

use crate::errors::{Error, Result};
use crate::messages::{self, AisMessage};
use nom::branch::alt;
use nom::bytes::complete::{tag, take, take_until};
use nom::character::complete::{anychar, digit1};
use nom::combinator::{map, map_res, opt, peek, verify};
use nom::number::complete::hex_u32;
use nom::sequence::{delimited, terminated};
use nom::IResult;

pub const MAX_SENTENCE_SIZE_BYTES: usize = 384;

#[cfg(any(feature = "std", feature = "alloc"))]
pub type AisRawData = lib::std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type AisRawData = lib::std::vec::Vec<u8, MAX_SENTENCE_SIZE_BYTES>;

#[derive(PartialEq, Eq, Debug)]
/// Represents the NMEA sentence type of an AIS message
pub enum AisReportType {
    /// Report from another ship
    VDM,
    /// Report from own ship
    VDO,
    /// Unknown report type
    Unknown,
}

impl<'a> From<&'a [u8]> for AisReportType {
    fn from(typ: &'a [u8]) -> Self {
        match typ {
            b"VDM" => Self::VDM,
            b"VDO" => Self::VDO,
            _ => Self::Unknown,
        }
    }
}

/// Talker ID for the AIS station
#[derive(PartialEq, Eq, Debug)]
pub enum TalkerId {
    /// NMEA 4.0 Base AIS station
    AB,
    /// NMEA 4.0 Dependent AIS Base station
    AD,
    /// Mobile AIS station
    AI,
    /// NMEA 4.0 Aid to Navigation AIS station
    AN,
    /// NMEA 4.0 AIS Receiving station
    AR,
    /// NMEA 4.0 Limited Base station
    AS,
    /// NMEA 4.0 AIS Transmitting station
    AT,
    /// NMEA 4.0 Repeater AIS station
    AX,
    /// Base AIS station (deprecated)
    BS,
    /// NMEA 4.0 Physical Shore AIS station
    SA,
    /// Unknown talker ID
    Unknown,
}
impl<'a> From<&'a [u8]> for TalkerId {
    fn from(talker_id: &'a [u8]) -> Self {
        match talker_id {
            b"AB" => Self::AB,
            b"AD" => Self::AD,
            b"AI" => Self::AI,
            b"AN" => Self::AN,
            b"AR" => Self::AR,
            b"AS" => Self::AS,
            b"AT" => Self::AT,
            b"AX" => Self::AX,
            b"BS" => Self::BS,
            b"SA" => Self::SA,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AisFragments {
    Complete(AisSentence),
    Incomplete(AisSentence),
}

impl From<AisFragments> for Option<AisSentence> {
    fn from(frag: AisFragments) -> Self {
        match frag {
            AisFragments::Complete(sentence) => Some(sentence),
            AisFragments::Incomplete(_) => None,
        }
    }
}

impl From<AisFragments> for Result<AisSentence> {
    fn from(frag: AisFragments) -> Self {
        match frag {
            AisFragments::Complete(sentence) => Ok(sentence),
            AisFragments::Incomplete(_) => Err("Incomplete message".into()),
        }
    }
}

#[derive(Debug, Default)]
pub struct AisParser {
    message_id: Option<u8>,
    fragment_number: u8,
    data: AisRawData,
}

impl AisParser {
    /// Creates a new `AisParser` instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `line` as an NMEA sentence, checking the checksum and returning an
    /// an `AisSentence`. Note that several `AisSentence`s might be required to
    /// complete a message, if they are fragments
    /// If `decode` is `true`, the internal AIS message will also be parsed
    /// If it is `false`, then internal AIS messages will be ignored.
    /// In both cases, AIS data will be passed along raw.
    pub fn parse(&mut self, line: &[u8], decode: bool) -> Result<AisFragments> {
        let (_, (data, mut ais_sentence, checksum)) = parse_nmea_sentence(line)?;
        Self::check_checksum(data, checksum)?;
        if ais_sentence.has_more() {
            if ais_sentence.fragment_number == 1 {
                self.message_id = ais_sentence.message_id;
                self.fragment_number = 0;
                self.data = AisRawData::default();
            }
            self.verify_and_extend_data(&ais_sentence)?;
            Ok(AisFragments::Incomplete(ais_sentence))
        } else {
            if ais_sentence.is_fragment() {
                self.verify_and_extend_data(&ais_sentence)?;
                let mut data = AisRawData::default();
                lib::std::mem::swap(&mut data, &mut self.data);
                ais_sentence.data = data;
            }
            if decode {
                let unarmored =
                    messages::unarmor(&ais_sentence.data, ais_sentence.fill_bit_count as usize)?;
                ais_sentence.message = Some(messages::parse(&unarmored)?)
            }
            Ok(AisFragments::Complete(ais_sentence))
        }
    }

    fn verify_and_extend_data(&mut self, ais_sentence: &AisSentence) -> Result<()> {
        if self.message_id != ais_sentence.message_id {
            return Err("Message ID out of sequence".into());
        }
        if ais_sentence.fragment_number - self.fragment_number != 1 {
            return Err("Fragment numbers out of sequence".into());
        }
        self.fragment_number = ais_sentence.fragment_number;
        #[cfg(any(feature = "std", feature = "alloc"))]
        self.data.extend_from_slice(&ais_sentence.data);
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        self.data
            .extend_from_slice(&ais_sentence.data)
            .map_err(|_| Error::from("Vec is full on extend_from_slice"))?;
        Ok(())
    }

    /// Verifies the AIS sentence checksum
    fn check_checksum(sentence: &[u8], expected_checksum: u8) -> Result<u8> {
        let received_checksum = sentence.iter().fold(0u8, |acc, &item| acc ^ item);
        if expected_checksum != received_checksum {
            Err(Error::Checksum {
                expected: expected_checksum,
                found: received_checksum,
            })
        } else {
            Ok(received_checksum)
        }
    }
}

#[derive(Debug, PartialEq)]
/// Represents an NMEA sentence parsed as AIS
pub struct AisSentence {
    pub talker_id: TalkerId,
    pub report_type: AisReportType,
    pub num_fragments: u8,
    pub fragment_number: u8,
    pub message_id: Option<u8>,
    pub channel: Option<char>,
    pub data: AisRawData,
    pub fill_bit_count: u8,
    pub message_type: u8,
    pub message: Option<AisMessage>,
}

impl AisSentence {
    /// Returns whether there are more fragments to come
    pub fn has_more(&self) -> bool {
        self.fragment_number < self.num_fragments
    }

    /// Returns whether this is part of a fragmented message set
    pub fn is_fragment(&self) -> bool {
        self.num_fragments != 1
    }
}

/// Converts bytes representing an ASCII number to a string slice
fn parse_numeric_string(data: &[u8]) -> IResult<&[u8], &str> {
    map_res(digit1, lib::std::str::from_utf8)(data)
}

/// Converts bytes representing an ASCII number to a u8
fn parse_u8_digit(data: &[u8]) -> IResult<&[u8], u8> {
    map_res(parse_numeric_string, lib::std::str::FromStr::from_str)(data)
}

/// Named parser for the AIS portion of an NMEA sentence
fn parse_ais_sentence(data: &[u8]) -> IResult<&[u8], AisSentence> {
    let (data, talker_id) = map(take(2u8), Into::into)(data)?;
    let (data, report_type) = map(take(3u8), Into::into)(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, num_fragments) = parse_u8_digit(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, fragment_number) = parse_u8_digit(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, message_id) = opt(parse_u8_digit)(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, channel_bytes) = take_until(",")(data)?;
    let (_, channel) = opt(anychar)(channel_bytes)?;
    let (data, _) = tag(",")(data)?;
    let (data, ais_data) = take_until(",")(data)?;
    let (data, _) = tag(",")(data)?;
    let (data, fill_bit_count) = verify(parse_u8_digit, |val| *val < 6)(data)?;
    let (_, message_type) = messages::message_type(ais_data)?;
    #[cfg(any(feature = "std", feature = "alloc"))]
    let ais_data_owned = ais_data.into();
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    let ais_data_owned = ais_data.try_into().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(
            ais_data,
            nom::error::ErrorKind::TooLarge,
        ))
    })?;
    Ok((
        data,
        AisSentence {
            talker_id,
            report_type,
            num_fragments,
            fragment_number,
            message_id,
            channel,
            data: ais_data_owned,
            fill_bit_count,
            message_type,
            message: None,
        },
    ))
}

/// Named parser for an overall NMEA 0183 sentence
fn parse_nmea_sentence(data: &[u8]) -> IResult<&[u8], (&[u8], AisSentence, u8)> {
    let (data, _) = opt(delimited(tag("\\"), take_until("\\"), tag("\\")))(data)?;
    let (data, _) = alt((tag("!"), tag("$")))(data)?;
    let (data, raw) = peek(take_until("*"))(data)?;
    let (data, msg) = terminated(parse_ais_sentence, tag("*"))(data)?;
    let (data, checksum) = verify(hex_u32, |val| val <= &0xff)(data)?;
    Ok((data, (raw, msg, checksum as u8)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_CHECKSUM: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A";
    const BAD_CHECKSUM: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const BAD_STRUCTURE: &[u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0,TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const FRAGMENT_1: &[u8] =
        b"!AIVDM,2,1,1,B,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP00000,0*78";
    const FRAGMENT_2: &[u8] = b"!AIVDM,2,2,1,B,0000000,2*26";
    const NO_CHANNEL: &[u8] = b"!AIVDM,1,1,,,34RvgN500005tLTMfjiTs3u`0>`<,0*7A";
    const AIS_START_IDX: usize = 14;
    const AIS_END_IDX: usize = 61;
    const WITH_TAG_BLOCK: &[u8] =
        b"\\s:2573345,c:1696241893*00\\!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A";
    const WITH_BAD_TAG_BLOCK: &[u8] =
        b"s:2573345,c:1696241893*00\\!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A";

    #[test]
    fn parse_valid_structure() {
        let result = parse_ais_sentence(&GOOD_CHECKSUM[1..63]).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(
            result.1,
            AisSentence {
                talker_id: TalkerId::AI,
                report_type: AisReportType::VDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: Some('A'),
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX].into(),
                fill_bit_count: 0,
                message_type: 17,
                message: None,
            }
        );
    }

    #[test]
    fn parse_invalid_structure() {
        assert!(parse_ais_sentence(&BAD_STRUCTURE[1..64]).is_err());
    }

    #[test]
    fn parse_valid_sentence() {
        let result = parse_nmea_sentence(GOOD_CHECKSUM).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(
            (result.1).1,
            AisSentence {
                talker_id: TalkerId::AI,
                report_type: AisReportType::VDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: Some('A'),
                #[cfg(any(feature = "std", feature = "alloc"))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX].into(),
                #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX]
                    .try_into()
                    .unwrap(),
                fill_bit_count: 0,
                message_type: 17,
                message: None,
            }
        );
        assert_eq!((result.1).2, 122);
    }

    #[test]
    fn parse_using_struct_valid() {
        let mut parser = AisParser::new();
        let result = parser.parse(GOOD_CHECKSUM, false).unwrap();
        assert_eq!(
            result,
            AisFragments::Complete(AisSentence {
                talker_id: TalkerId::AI,
                report_type: AisReportType::VDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: Some('A'),
                #[cfg(any(feature = "std", feature = "alloc"))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX].into(),
                #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX]
                    .try_into()
                    .unwrap(),
                fill_bit_count: 0,
                message_type: 17,
                message: None,
            })
        );
    }

    #[test]
    fn parse_valid_checksum() {
        let mut parser = AisParser::new();
        let result = parser.parse(GOOD_CHECKSUM, false);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_checksum() {
        let mut parser = AisParser::new();
        let result = parser.parse(BAD_CHECKSUM, false);
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiple_fragments() {
        let mut parser = AisParser::new();
        let frag1 = parser.parse(FRAGMENT_1, false).unwrap();
        let frag2 = parser.parse(FRAGMENT_2, false).unwrap();
        if let AisFragments::Complete(_) = frag1 {
            panic!("Expected frag1 to be incomplete, but it was {:?}", frag1);
        }
        if let AisFragments::Complete(complete) = frag2 {
            assert!(complete.num_fragments > 1);
        } else {
            panic!("Expected frag2 to be complete, but it was {:?}", frag2);
        }
    }

    #[test]
    fn test_talker_id_conversions() {
        assert_eq!(TalkerId::from(b"AI".as_ref()), TalkerId::AI);
        assert_eq!(TalkerId::from(b"AB".as_ref()), TalkerId::AB);
        assert_eq!(TalkerId::from(b"BS".as_ref()), TalkerId::BS);
    }

    #[test]
    fn test_no_channel() {
        let result = parse_nmea_sentence(NO_CHANNEL).unwrap();
        let sentence = (result.1).1;
        assert_eq!(sentence.channel, None);
    }

    #[test]
    fn parse_valid_sentence_with_tag_block() {
        let result = parse_nmea_sentence(WITH_TAG_BLOCK).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(
            (result.1).1,
            AisSentence {
                talker_id: TalkerId::AI,
                report_type: AisReportType::VDM,
                num_fragments: 1,
                fragment_number: 1,
                message_id: None,
                channel: Some('A'),
                #[cfg(any(feature = "std", feature = "alloc"))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX].into(),
                #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
                data: GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX]
                    .try_into()
                    .unwrap(),
                fill_bit_count: 0,
                message_type: 17,
                message: None,
            }
        );
        assert_eq!((result.1).2, 122);
    }

    #[test]
    fn parse_sentence_with_invalid_tag_block() {
        assert!(parse_ais_sentence(&WITH_BAD_TAG_BLOCK[1..64]).is_err());
    }
}
