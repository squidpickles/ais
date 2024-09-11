use crate::errors::Error;
use crate::messages::tag_block::TagBlock;
use crate::messages::AisMessage;
use crate::sentence::{AisFragments, AisParser};
use crate::lib::std::error::Error as StdError;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};
/// Parses a single line of NMEA data using the provided AIS parser.
///
/// This function attempts to parse a given NMEA line into a tag block and an AIS message,
/// printing the results to the console.
///
/// # Arguments
/// * `parser` - The AIS parser to use.
/// * `line` - A byte slice containing the NMEA line to parse.
///

async fn parse_nmea_line(parser: &mut AisParser, line: &[u8]) {
    // Convert the line to a string
    let line_str = std::str::from_utf8(line).expect("Invalid UTF-8 sequence");

    // Print the received message
    println!("Received message: {}", line_str);

    // Check for a tag block by looking for the start of a NMEA sentence ('!')
    if let Some(nmea_start_idx) = line_str.find('!') {
        // Extract the tag block (everything before the '!') and the NMEA sentence
        let tag_block_str = &line_str[..nmea_start_idx];
        let nmea_sentence = &line_str[nmea_start_idx..];

        // Check if there's a valid tag block (should start and end with '\')
        if tag_block_str.starts_with('\\') && tag_block_str.ends_with('\\') {
            // Remove the leading and trailing backslashes from the tag block
            let tag_block_content = &tag_block_str[1..tag_block_str.len() - 1];

            // Parse the tag block
            match TagBlock::parse(tag_block_content) {
                Ok(Some(tag_block)) => {
                    println!("Parsed TagBlock: {:?}", tag_block);
                }
                Ok(None) => {
                    println!("No tag block found");
                }
                Err(err) => {
                    eprintln!("Error parsing tag block: {}", err);
                    return;
                }
            }
        }

        // Parse the NMEA sentence
        match parser.parse(nmea_sentence.as_bytes(), true) {
            Ok((_, AisFragments::Complete(sentence))) => {
                println!(
                    "Parsed NMEA Sentence: {:?}\nMessage: {:?}",
                    nmea_sentence, sentence.message
                );
            }
            Err(err) => {
                eprintln!("Error parsing line {:?}: {:?}", nmea_sentence, err);
            }
            _ => {}
        }

        // Print separator between messages
        println!("*************************");
    } else {
        eprintln!("No valid NMEA sentence found in line");
    }
}

/// Decodes a stream of AIS messages from UDP.
///
/// This function binds to the given UDP address and decodes incoming AIS messages, printing the results to the console.
///
/// # Arguments
/// * `address` - The address to bind to in the form "ip:port".
///

pub async fn decode_from_udp(address: &str) -> Result<(), Box<dyn StdError>> {
    let socket = UdpSocket::bind(address).await?;
    let mut buf = [0; 1024];
    let mut parser = AisParser::new();

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;
        parse_nmea_line(&mut parser, &buf[..len]).await;
    }
}

/// Decodes a stream of AIS messages from TCP.
///
/// This function connects to the given TCP address and decodes incoming AIS messages,
/// printing the results to the console.
///
/// # Arguments
/// * `address` - The address to connect to in the form "ip:port".
///
pub async fn decode_from_tcp(address: &str) -> Result<(), Box<dyn StdError>> {
    let stream = TcpStream::connect(address).await?;
    let mut parser = AisParser::new();
    let mut reader = BufReader::new(stream);
    let mut line = Vec::new();

    while reader.read_until(b'\n', &mut line).await? != 0 {
        parse_nmea_line(&mut parser, &line).await;
        line.clear();
    }

    Ok(())
}

/// Decodes a file of AIS messages.
///
/// This function reads AIS messages from a file and decodes them, printing the results to the console.
///
/// # Arguments
/// * `path` - The path to the file containing AIS messages.
///
///
pub async fn decode_from_file(path: &str) -> Result<(), Box<dyn StdError>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut parser = AisParser::new();

    while let Some(line) = lines.next_line().await? {
        parse_nmea_line(&mut parser, line.as_bytes()).await;
    }

    Ok(())
}

/// Decodes a single AIS message.
///
/// This function parses a single AIS message from a byte slice and returns the parsed message.
///
/// # Arguments
/// * `message` - A byte slice containing the AIS message to parse.
///
/// # Returns
/// * A result containing the parsed AIS message or an error.
///
/// # Errors
/// * Returns an error if the message is incomplete or cannot be parsed.
///

pub fn decode(message: &[u8]) -> Result<AisMessage, Error> {
    let mut parser = AisParser::new();
    match parser.parse(message, true)? {
        (Some(tag_block), AisFragments::Complete(sentence)) => {
            println!("TagBlock: {:?}", tag_block);
            sentence.message.ok_or(Error::Nmea {
                msg: "Incomplete message".into(),
            })
        }
        (None, AisFragments::Complete(sentence)) => sentence.message.ok_or(Error::Nmea {
            msg: "Incomplete message".into(),
        }),
        _ => Err(Error::Nmea {
            msg: "Incomplete message".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::position_report::NavigationStatus;
    use tempfile;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, UdpSocket};

    // Function to validate PositionReport messages
    fn validate_position_report(report: &crate::messages::position_report::PositionReport) {
        assert_eq!(report.message_type, 1);
        assert_eq!(report.mmsi, 367380120);
        assert_eq!(
            report.navigation_status,
            Some(NavigationStatus::UnderWayUsingEngine)
        );
        assert_eq!(report.speed_over_ground, Some(0.1));
        assert_eq!(report.longitude, Some(-122.404335));
        assert_eq!(report.latitude, Some(37.806946));
        assert_eq!(report.course_over_ground, Some(245.2));
        assert_eq!(report.timestamp, 59);
        assert!(report.raim);
    }

    #[tokio::test]
    async fn test_parse_nmea_line() {
        let mut parser = AisParser::new();
        let line = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05";

        parse_nmea_line(&mut parser, line).await;

        if let Ok((_, AisFragments::Complete(sentence))) = parser.parse(line, true) {
            if let Some(AisMessage::PositionReport(ref report)) = sentence.message {
                validate_position_report(report);
            } else {
                panic!("Failed to parse message as PositionReport");
            }
        } else {
            panic!("Failed to parse NMEA line");
        }
    }

    #[tokio::test]
    async fn test_decode_from_udp() {
        let address = "127.0.0.1:12345";
        let test_data = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05";

        let server_handle = tokio::spawn(async move {
            decode_from_udp(address).await.unwrap();
        });

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(test_data, address).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut parser = AisParser::new();
        if let Ok((_, AisFragments::Complete(sentence))) = parser.parse(test_data, true) {
            if let Some(AisMessage::PositionReport(ref report)) = sentence.message {
                validate_position_report(report);
            } else {
                panic!("Failed to parse message as PositionReport");
            }
        } else {
            panic!("Failed to parse NMEA line");
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_decode_from_tcp() {
        let address = "127.0.0.1:12346";
        let listener = TcpListener::bind(address).await.unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let test_data = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05\n";
            socket.write_all(test_data).await.unwrap();
        });

        decode_from_tcp(address).await.unwrap();

        let message = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05";
        let mut parser = AisParser::new();
        if let Ok((_, AisFragments::Complete(sentence))) = parser.parse(message, true) {
            if let Some(AisMessage::PositionReport(ref report)) = sentence.message {
                validate_position_report(report);
            } else {
                panic!("Failed to parse message as PositionReport");
            }
        } else {
            panic!("Failed to parse NMEA line");
        }
    }

    #[tokio::test]
    async fn test_decode_from_file() {
        let test_data = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05\n";
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        tokio::fs::write(&file_path, test_data).await.unwrap();

        decode_from_file(file_path.to_str().unwrap()).await.unwrap();

        let mut parser = AisParser::new();
        if let Ok((_, AisFragments::Complete(sentence))) = parser.parse(test_data, true) {
            if let Some(AisMessage::PositionReport(ref report)) = sentence.message {
                validate_position_report(report);
            } else {
                panic!("Failed to parse message as PositionReport");
            }
        } else {
            panic!("Failed to parse NMEA line");
        }
    }

    #[test]
    fn test_decode() {
        let message = b"!AIVDM,1,1,,B,15NG6V0P01G?cFhE`R2IU?wn28R>,0*05";
        let result = decode(message);

        match result {
            Ok(AisMessage::PositionReport(ref report)) => {
                validate_position_report(report);
            }
            _ => panic!("Failed to decode the message correctly"),
        }
    }
}
