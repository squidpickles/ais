//! Handlers for AIS messages at the NMEA sentence layer
use nom::{anychar, digit, IResult, hex_u32};
use errors::*;

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
        let (data, ais_msg, checksum) = match nmea_sentence(line) {
            IResult::Done(_, result) => result,
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS sentence")?,
            IResult::Incomplete(_) => Err("incomplete AIS sentence")?,
        };
        Self::check_checksum(data, checksum)?;
        Ok(ais_msg)
    }

    /// Verifies the checksum
    fn check_checksum(sentence: &[u8], expected_checksum: u8) -> Result<u8> {
        let received_checksum = sentence.iter().fold(0u8, |acc, &item| acc ^ item);
        if expected_checksum != received_checksum {
            Err(ErrorKind::Checksum(expected_checksum, received_checksum))?
        } else {
            Ok(received_checksum)
        }
    }

    /// Returns whether there are more fragments to come
    pub fn has_more(&self) -> bool {
        self.fragment_number < self.num_fragments
    }
}

named!(numeric_string<&str>, map_res!(digit, ::std::str::from_utf8));

named!(
    u8_digit<u8>,
    map_res!(numeric_string, ::std::str::FromStr::from_str)
);
named!(nmea_start, alt!(tag!("!") | tag!("$")));
named!(ais_type, alt!(tag!("AIVDM") | tag!("AIVDO")));
named!(num_fragments<u8>, call!(u8_digit));
named!(fragment_number<u8>, call!(u8_digit));
named!(sequential_message_id<Option<u8>>, opt!(u8_digit));
named!(channel<char>, call!(anychar));
named!(ais_data, take_until!(","));
named!(fill_bit_count<u8>, verify!(u8_digit, |val: u8| val < 6));
named!(data_end, tag!("*"));
named!(checksum<u32>, verify!(hex_u32, |val: u32| val <= 0xff));
named!(pub ais_sentence<AisSentence>, do_parse!(
    typ: ais_type
    >> tag!(",")
    >> ns: num_fragments
    >> tag!(",")
    >> sn: fragment_number
    >> tag!(",")
    >> smid: sequential_message_id
    >> tag!(",")
    >> chan: channel
    >> tag!(",")
    >> data: ais_data
    >> tag!(",")
    >> fb: fill_bit_count
    >> (AisSentence {msg_type: typ.into(), num_fragments: ns, fragment_number: sn, message_id: smid, channel: chan, data: data, fill_bit_count: fb})
));

named!(pub nmea_sentence<(&[u8], AisSentence, u8)>, do_parse!(
    nmea_start
    >> raw: peek!(take_until!("*"))
    >> msg: terminated!(ais_sentence, data_end)
    >> cs: checksum
    >> (raw, msg, cs as u8)
));

#[cfg(test)]
mod tests {
    const GOOD_CHECKSUM: &'static [u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A";
    const BAD_CHECKSUM: &'static [u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const BAD_STRUCTURE: &'static [u8] =
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0,TahI7@@;V4=v:nv;h00003vP100,0*8D";
    const AIS_START_IDX: usize = 14;
    const AIS_END_IDX: usize = 61;
    use super::*;
    use nom;

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
        let result = ais_sentence(&BAD_STRUCTURE[1..64]).unwrap_err();
        assert_eq!(result, nom::ErrorKind::Digit);
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
