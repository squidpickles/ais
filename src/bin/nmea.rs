use ais::sentence::{AisFragments, AisParser};
use std::io::BufRead;

use std::io;

fn parse_nmea_line(parser: &mut AisParser, line: &[u8]) -> Result<(), ais::errors::Error> {
    let sentence = parser.parse(line, true)?;
    if let AisFragments::Complete(sentence) = sentence {
        println!(
            "{:?}\t{:?}",
            std::str::from_utf8(line).unwrap(),
            sentence.message
        );
    }
    Ok(())
}

fn main() {
    let mut parser = AisParser::new();
    let stdin = io::stdin();
    {
        let handle = stdin.lock();

        handle
            .split(b'\n')
            .map(|line| line.unwrap())
            .for_each(|line| {
                parse_nmea_line(&mut parser, &line).unwrap_or_else(|err| {
                    eprintln!("{:?}\t{:?}", std::str::from_utf8(&line).unwrap(), err);
                });
            });
    }
}
