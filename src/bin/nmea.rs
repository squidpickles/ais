extern crate ais;

use ais::messages;
use ais::sentence::AisSentence;
use std::io::BufRead;

use std::io;

fn parse_nmea_line(line: &[u8]) -> Result<(), ais::errors::Error> {
    let sentence = AisSentence::parse(line)?;
    let raw = messages::unarmor(sentence.data, sentence.fill_bit_count as usize)?;
    let message = messages::parse(&raw)?;
    println!("{:?}", message);
    Ok(())
}

fn main() {
    let stdin = io::stdin();
    {
        let handle = stdin.lock();

        handle
            .split(b'\n')
            .map(|line| line.unwrap())
            .for_each(|line| {
                parse_nmea_line(&line).unwrap_or(());
            });
    }
}
