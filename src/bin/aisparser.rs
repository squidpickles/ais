use ais::{decode, decode_from_file, decode_from_tcp, decode_from_udp};
use clap::{Arg, Command};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("ais-decode")
        .version("1.0")
        .about("AIS message decoding")
        .arg(
            Arg::new("udp")
                .short('u')
                .long("udp")
                .value_name("ADDRESS")
                .help("Address to listen for UDP messages"),
        )
        .arg(
            Arg::new("tcp")
                .short('t')
                .long("tcp")
                .value_name("ADDRESS")
                .help("Address to connect for TCP messages"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("PATH")
                .help("Path to the file to read AIS messages from"),
        )
        .arg(
            Arg::new("message")
                .short('m')
                .long("message")
                .value_name("AIS_MESSAGE")
                .help("A single AIS message to decode"),
        )
        .get_matches();

    if let Some(address) = matches.get_one::<String>("udp") {
        decode_from_udp(address).await?;
    } else if let Some(address) = matches.get_one::<String>("tcp") {
        decode_from_tcp(address).await?;
    } else if let Some(path) = matches.get_one::<String>("file") {
        decode_from_file(path).await?;
    } else if let Some(message) = matches.get_one::<String>("message") {
        match decode(message.as_bytes()) {
            Ok(parsed_message) => println!("Parsed AIS Message: {:?}", parsed_message),
            Err(e) => eprintln!("Failed to parse AIS message: {}", e),
        }
    } else {
        eprintln!("No valid command provided.");
    }

    Ok(())
}
