use ais::lib;
use ais::sentence::{AisFragments, AisParser, AisSentence};
use lib::std::io::{self, BufRead};

fn parse_nmea_line_to_json(parser: &mut AisParser, line: &[u8]) -> Result<(), ais::errors::Error> {
    let sentence = parser.parse(line, true)?;
    if let AisFragments::Complete(sentence) = sentence {
        match serialize_to_json(&sentence) {
            Ok(json) => println!("{}", json),
            Err(err) => eprintln!("Error serializing to JSON: {}", err),
        }
    }
    Ok(())
}

pub fn serialize_to_json(sentence: &AisSentence) -> serde_json::Result<String> {
    serde_json::to_string(sentence)
}

pub fn deserialize_from_json(json_data: &str) -> serde_json::Result<AisSentence> {
    serde_json::from_str(json_data)
}

fn main() {
    let mut parser = AisParser::new();
    let stdin = io::stdin();
    let handle = stdin.lock();

    handle
        .split(b'\n')
        .map(|line| line.unwrap())
        .for_each(|line| {
            parse_nmea_line_to_json(&mut parser, &line).unwrap_or_else(|err| {
                eprintln!(
                    "Error parsing line: {:?}\t{:?}",
                    lib::std::str::from_utf8(&line).unwrap(),
                    err
                );
            });
        });
}
