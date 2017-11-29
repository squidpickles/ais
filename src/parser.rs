use nom::{hex_u32, anychar, digit, IResult};
use errors::*;

#[derive(PartialEq, Debug)]
pub enum AisMessageType {
    AIVDM,
    AIVDO,
    Unknown,
}

impl<'a> From<&'a [u8]> for AisMessageType {
    fn from(typ: &'a [u8]) -> Self {
        match typ {
            b"AIVDM" => AisMessageType::AIVDM,
            b"AIVDO" => AisMessageType::AIVDO,
            _ => AisMessageType::Unknown,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct AisMessage<'a> {
    pub msg_type: AisMessageType,
    pub num_fragments: u8,
    pub fragment_number: u8,
    pub message_id: Option<u8>,
    pub channel: char,
    raw_data: &'a [u8],
    fill_bit_count: u8,
}

impl<'a> AisMessage<'a> {
    pub fn parse(line: &'a [u8]) -> Result<AisMessage<'a>> {
        let (raw_data, ais_msg, checksum) = match nmea_sentence(line) {
            IResult::Done(_, result) => result,
            IResult::Error(err) => Err(err).chain_err(|| "parsing AIS message")?,
            IResult::Incomplete(_) => Err("incomplete AIS message")?,
        };
        Self::check_checksum(raw_data, checksum)?;
        Ok(ais_msg)
    }

    fn check_checksum(message: &[u8], expected_checksum: u8) -> Result<u8> {
        let received_checksum = message.iter().fold(0u8, |acc, &item| acc ^ item);
        if expected_checksum != received_checksum {
            Err(ErrorKind::Checksum(expected_checksum, received_checksum))?
        } else {
            Ok(received_checksum)
        }
    }
}

named!(numeric_string<&str>,
    map_res!(
        digit,
        ::std::str::from_utf8
    )
);

named!(u8_digit<u8>, map_res!(numeric_string, ::std::str::FromStr::from_str));
named!(nmea_start, alt!(tag!("!") | tag!("$")));
named!(ais_type, alt!(tag!("AIVDM") | tag!("AIVDO")));
named!(num_fragments<u8>, call!(u8_digit));
named!(fragment_number<u8>, call!(u8_digit));
named!(sequential_message_id<Option<u8> >, opt!(u8_digit));
named!(channel<char>, call!(anychar));
named!(ais_data, take_until!(","));
named!(fill_bit_count<u8>, verify!(u8_digit, |val:u8| val < 6));
named!(data_end, tag!("*"));
named!(checksum<u32>, verify!(hex_u32, |val:u32| val <= 0xff));
named!(pub ais_message<AisMessage>, do_parse!(
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
    >> (AisMessage {msg_type: typ.into(), num_fragments: ns, fragment_number: sn, message_id: smid, channel: chan, raw_data: data, fill_bit_count: fb})
));

named!(pub nmea_sentence<(&[u8], AisMessage, u8)>, do_parse!(
    nmea_start
    >> raw: peek!(take_until!("*"))
    >> msg: terminated!(ais_message, data_end)
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
        let result = ais_message(&GOOD_CHECKSUM[1..63]).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!(result.1, AisMessage {msg_type: AisMessageType::AIVDM, num_fragments: 1, fragment_number: 1, message_id: None, channel: 'A', raw_data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX], fill_bit_count: 0});
    }

    #[test]
    fn parse_invalid_structure() {
        let result = ais_message(&BAD_STRUCTURE[1..64]).unwrap_err();
        assert_eq!(result, nom::ErrorKind::Digit);
    }

    #[test]
    fn parse_valid_sentence() {
        let result = nmea_sentence(GOOD_CHECKSUM).unwrap();
        assert_eq!(result.0, b"");
        assert_eq!((result.1).1, AisMessage {msg_type: AisMessageType::AIVDM, num_fragments: 1, fragment_number: 1, message_id: None, channel: 'A', raw_data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX], fill_bit_count: 0});
        assert_eq!((result.1).2, 122);
    }

    #[test]
    fn parse_using_struct_valid() {
        let result = AisMessage::parse(GOOD_CHECKSUM).unwrap();
        assert_eq!(result, AisMessage {msg_type: AisMessageType::AIVDM, num_fragments: 1, fragment_number: 1, message_id: None, channel: 'A', raw_data: &GOOD_CHECKSUM[AIS_START_IDX..AIS_END_IDX], fill_bit_count: 0});
    }

    #[test]
    fn parse_valid_checksum() {
        let result = AisMessage::parse(GOOD_CHECKSUM);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_invalid_checksum() {
        let result = AisMessage::parse(BAD_CHECKSUM);
        assert!(result.is_err());
    }

}
