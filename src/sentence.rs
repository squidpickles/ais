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

/// Internal slot for tracking reassembly of a single multi-fragment message
#[derive(Debug, Default)]
struct FragmentSlot {
    fragment_number: u8,
    data: AisRawData,
}

const NUM_MESSAGE_SLOTS: usize = 10; // 10 possible message IDs, so this allocates one per ID

/// Stateful parser for NMEA 0183 AIS sentences with automatic multi-fragment
/// reassembly. Use [`Self::parse_message()`], [`Self::parse()`], or [`Self::parse_raw()`]
/// as needed. Note that fragments must be received in order, and if there
/// are multiple senders simultaneously sending messages with the same ID,
/// collisions will occur.
#[derive(Debug)]
pub struct AisParser {
    slots: [Option<FragmentSlot>; NUM_MESSAGE_SLOTS],
}

impl Default for AisParser {
    fn default() -> Self {
        Self {
            slots: core::array::from_fn(|_| None),
        }
    }
}

impl AisParser {
    /// Creates a new [`AisParser`] instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `line` as an AIS message. If the checksum matches and the message
    /// is complete (i.e. it is not a fragment), and the message type is also
    /// supported, this will return the message itself. Does not return the NMEA
    /// sentence information or raw AIS data; if that's needed, use the lower-level
    /// [`AisParser::parse()`] method instead. Unsupported message types will return
    /// [`Error::UnsupportedMessageType`].
    pub fn parse_message(&mut self, line: &[u8]) -> Result<Option<AisMessage>> {
        match self.parse(line)? {
            AisFragments::Complete(sentence) => Ok(sentence.message),
            AisFragments::Incomplete(_) => Ok(None),
        }
    }

    /// Parses `line` as an NMEA sentence, checking the checksum and returning an
    /// an [`AisSentence`]. Note that several [`AisSentence`]s might be required to
    /// complete a message, if they are fragments.
    /// The internal AIS message will also be parsed, and AIS data will additionally
    /// be passed along raw. See [`AisParser::parse_raw()`] if you don't need the message
    /// parsing. See [`AisParser::parse_message()`] if you only want the message, for a
    /// higher-level interface.
    /// Unsupported message types will return [`Error::UnsupportedMessageType`].
    pub fn parse(&mut self, line: &[u8]) -> Result<AisFragments> {
        let mut result = self.parse_raw(line)?;
        if let AisFragments::Complete(ref mut ais_sentence) = result {
            let unarmored =
                messages::unarmor(&ais_sentence.data, ais_sentence.fill_bit_count as usize)?;
            ais_sentence.message = Some(messages::parse(&unarmored)?);
        }
        Ok(result)
    }

    /// Parses `line` as an NMEA sentence, checking the checksum and returning an
    /// an [`AisSentence`]. Note that several [`AisSentence`]s might be required to
    /// complete a message, if they are fragments. AIS data will be passed along raw.
    pub fn parse_raw(&mut self, line: &[u8]) -> Result<AisFragments> {
        let (_, (data, mut ais_sentence, checksum)) = parse_nmea_sentence(line)?;
        Self::check_checksum(data, checksum)?;

        if !ais_sentence.is_fragment() {
            return Ok(AisFragments::Complete(ais_sentence));
        }

        let idx = self.slot_index(&ais_sentence)?;

        if ais_sentence.fragment_number == 1 {
            self.slots[idx] = Some(FragmentSlot::default());
        }

        self.extend_slot(idx, &ais_sentence)?;

        if ais_sentence.has_more() {
            Ok(AisFragments::Incomplete(ais_sentence))
        } else {
            let slot = self.slots[idx]
                .take()
                .ok_or_else(|| Error::from("Missing fragment slot data"))?;
            ais_sentence.data = slot.data;
            Ok(AisFragments::Complete(ais_sentence))
        }
    }

    /// Maps a sentence's `message_id` to the corresponding slot index.
    /// Returns an error if the sentence is a fragment but has no `message_id`,
    /// or if the `message_id` is out of range.
    fn slot_index(&self, sentence: &AisSentence) -> Result<usize> {
        let idx = sentence
            .message_id
            .ok_or_else(|| Error::from("Fragment missing message ID"))
            .map(|id| id as usize)?;
        if (0..NUM_MESSAGE_SLOTS).contains(&idx) {
            Ok(idx)
        } else {
            Err(Error::from("Message ID out of range"))
        }
    }

    /// Validates fragment sequence and extends the slot with the sentence's data.
    fn extend_slot(&mut self, idx: usize, sentence: &AisSentence) -> Result<()> {
        let slot_option = &mut self.slots[idx];

        let expected_fragment_number = slot_option
            .as_ref()
            .map(|s| s.fragment_number + 1)
            .unwrap_or(1);

        if sentence.fragment_number != expected_fragment_number {
            *slot_option = None;
            return Err(Error::from("Fragment numbers out of sequence"));
        }

        let slot = slot_option
            .as_mut()
            .ok_or_else(|| Error::from("Fragment slot unexpectedly missing"))?;

        slot.fragment_number = sentence.fragment_number;

        #[cfg(any(feature = "std", feature = "alloc"))]
        slot.data.extend_from_slice(&sentence.data);

        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            if slot.data.extend_from_slice(&sentence.data).is_err() {
                *slot_option = None;
                return Err(Error::from("Vec is full on extend_from_slice"));
            }
        }

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
        let result = parser.parse_raw(GOOD_CHECKSUM).unwrap();
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
        let result = parser.parse_raw(GOOD_CHECKSUM);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_checksum() {
        let mut parser = AisParser::new();
        let result = parser.parse_raw(BAD_CHECKSUM);
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiple_fragments() {
        let mut parser = AisParser::new();
        let frag1 = parser.parse_raw(FRAGMENT_1).unwrap();
        let frag2 = parser.parse_raw(FRAGMENT_2).unwrap();
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

    #[test]
    fn parse_interleaved_fragments() {
        // Two different multi-fragment messages with different message IDs interleaved
        // Message ID 1, Fragment 1 of 2
        const MSG1_FRAG1: &[u8] =
            b"!AIVDM,2,1,1,A,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP00000,0*7B";
        // Message ID 2, Fragment 1 of 2
        const MSG2_FRAG1: &[u8] =
            b"!AIVDM,2,1,2,B,55?MbV02>H97jKH4<F16Dr;0@T4@Dk0000000000017R@sEE6N0j`0000000,0*5F";
        // Message ID 1, Fragment 2 of 2
        const MSG1_FRAG2: &[u8] = b"!AIVDM,2,2,1,A,0000000,2*25";
        // Message ID 2, Fragment 2 of 2
        const MSG2_FRAG2: &[u8] = b"!AIVDM,2,2,2,B,0000000,2*25";

        let mut parser = AisParser::new();

        let result1 = parser.parse_raw(MSG1_FRAG1).unwrap();
        assert!(matches!(result1, AisFragments::Incomplete(_)));

        let result2 = parser.parse_raw(MSG2_FRAG1).unwrap();
        assert!(matches!(result2, AisFragments::Incomplete(_)));

        let result3 = parser.parse_raw(MSG1_FRAG2).unwrap();
        if let AisFragments::Complete(sentence) = result3 {
            assert_eq!(sentence.num_fragments, 2);
            assert_eq!(sentence.message_id, Some(1));
        } else {
            panic!("Expected msg1 to be complete, but it was {:?}", result3);
        }

        let result4 = parser.parse_raw(MSG2_FRAG2).unwrap();
        if let AisFragments::Complete(sentence) = result4 {
            assert_eq!(sentence.num_fragments, 2);
            assert_eq!(sentence.message_id, Some(2));
        } else {
            panic!("Expected msg2 to be complete, but it was {:?}", result4);
        }
    }

    #[test]
    fn parse_interleaved_error_isolation() {
        // Test that an error in one message_id doesn't affect another
        const MSG1_FRAG1: &[u8] =
            b"!AIVDM,2,1,1,A,53`soB8000010KSOW<0P4eDp4l6000000000000U0p<24t@P05H3S833CDP00000,0*7B";
        const MSG2_FRAG1: &[u8] =
            b"!AIVDM,2,1,2,B,55?MbV02>H97jKH4<F16Dr;0@T4@Dk0000000000017R@sEE6N0j`0000000,0*5F";
        // Out-of-sequence fragment for message ID 1 (fragment 3, but we expect 2)
        const MSG1_FRAG3_BAD: &[u8] = b"!AIVDM,2,3,1,A,0000000,2*24";
        const MSG2_FRAG2: &[u8] = b"!AIVDM,2,2,2,B,0000000,2*25";

        let mut parser = AisParser::new();

        let result1 = parser.parse_raw(MSG1_FRAG1).unwrap();
        assert!(matches!(result1, AisFragments::Incomplete(_)));

        let result2 = parser.parse_raw(MSG2_FRAG1).unwrap();
        assert!(matches!(result2, AisFragments::Incomplete(_)));

        let result3 = parser.parse_raw(MSG1_FRAG3_BAD);
        assert!(result3.is_err());

        let result4 = parser.parse_raw(MSG2_FRAG2).unwrap();
        if let AisFragments::Complete(sentence) = result4 {
            assert_eq!(sentence.num_fragments, 2);
            assert_eq!(sentence.message_id, Some(2));
        } else {
            panic!(
                "Expected msg2 to be complete despite msg1 error, but it was {:?}",
                result4
            );
        }
    }
}
