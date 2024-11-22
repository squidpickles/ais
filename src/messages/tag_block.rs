use std::result::Result;

/// Tag blocks are optional components of AIS messages that provide additional metadata
/// through key-value pairs. These key-value pairs are enclosed within backslashes and
/// followed by a checksum. The metadata can include timestamps, source and destination
/// stations, line counts, relative times, and text. The checksum is used to validate the
/// integrity of the tag block. For more details, refer to the official documentation:
/// [NMEA Tag Blocks](https://gpsd.gitlab.io/gpsd/AIVDM.html#_nmea_tag_blocks).
///
/// Fields:
/// - `receiver_timestamp`: UNIX time in seconds or milliseconds.
/// - `destination_station`: Destination station, up to 15 characters.
/// - `line_count`: Line count.
/// - `relative_time`: Relative time.
/// - `source_station`: Source station.
/// - `text`: Text string, up to 15 characters.
/// - `checksum`: Checksum for validation.
#[derive(Debug, PartialEq)]
pub struct TagBlock {
    pub receiver_timestamp: Option<u64>,
    pub destination_station: Option<String>,
    pub line_count: Option<u32>,
    pub relative_time: Option<u32>,
    pub source_station: Option<String>,
    pub text: Option<String>,
    pub checksum: u8,
}

impl TagBlock {
    /// Parses a tag block from a given input string.
    ///
    /// # Arguments
    /// * `input` - A string slice that holds the tag block to be parsed.
    ///
    /// # Returns
    /// * `Ok(Some(TagBlock))` if parsing is successful.
    /// * `Ok(None)` if the tag block is empty.
    /// * `Err(String)` if the tag block format is invalid or if there is a checksum mismatch.
    pub fn parse(input: &str) -> Result<Option<Self>, String> {
        // Remove leading and trailing backslashes
        let input = input.trim_matches('\\');
        let parts: Vec<&str> = input.split('*').collect();

        if parts.len() != 2 {
            return Err("Invalid tag block format: missing checksum".into());
        }

        let key_value_part = parts[0];
        let checksum_str = parts[1];

        // Ensure checksum string length is 2
        if checksum_str.len() != 2 {
            return Err("Invalid checksum format: checksum should be 2 characters".into());
        }

        // Parse the provided checksum
        let provided_checksum = u8::from_str_radix(checksum_str, 16).map_err(|_| {
            "Invalid checksum format: checksum is not a valid hexadecimal number".to_string()
        })?;

        // Calculate the checksum
        let calculated_checksum = calculate_checksum(key_value_part.as_bytes());
        if calculated_checksum != provided_checksum {
            return Err(format!(
                "Checksum mismatch: calculated {:#02X}, expected {:#02X}",
                calculated_checksum, provided_checksum
            ));
        }

        let mut tag_block = TagBlock {
            receiver_timestamp: None,
            destination_station: None,
            line_count: None,
            relative_time: None,
            source_station: None,
            text: None,
            checksum: provided_checksum,
        };

        // Parse key-value pairs
        for kv in key_value_part.split(',') {
            if kv.len() < 3 {
                continue;
            }

            let (key, value) = kv.split_at(2);
            let value = value.to_string();

            match key {
                "c:" => {
                    tag_block.receiver_timestamp = value.parse().ok();
                }
                "d:" => {
                    tag_block.destination_station = Some(value);
                }
                "n:" => {
                    tag_block.line_count = value.parse().ok();
                }
                "r:" => {
                    tag_block.relative_time = value.parse().ok();
                }
                "s:" => {
                    tag_block.source_station = Some(value);
                }
                "t:" => {
                    tag_block.text = Some(value);
                }
                _ => {
                    // Ignore unknown keys
                }
            }
        }

        Ok(Some(tag_block))
    }
}

/// Calculates the checksum for the provided data using XOR operation.
///
/// # Arguments
/// * `data` - A byte slice of data for which the checksum is to be calculated.
///
/// # Returns
/// * `u8` - The calculated checksum.
fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &item| acc ^ item)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tag_block(
        input: &str,
        expected_receiver_timestamp: Option<u64>,
        expected_source_station: Option<String>,
        expected_destination_station: Option<String>,
        expected_line_count: Option<u32>,
        expected_relative_time: Option<u32>,
        expected_text: Option<String>,
        expected_checksum: u8,
    ) {
        let tag_block = TagBlock::parse(input).unwrap().unwrap();

        assert_eq!(tag_block.receiver_timestamp, expected_receiver_timestamp);
        assert_eq!(tag_block.source_station, expected_source_station);
        assert_eq!(tag_block.destination_station, expected_destination_station);
        assert_eq!(tag_block.line_count, expected_line_count);
        assert_eq!(tag_block.relative_time, expected_relative_time);
        assert_eq!(tag_block.text, expected_text);
        assert_eq!(tag_block.checksum, expected_checksum);
    }

    #[test]
    fn parse_tag_block_with_valid_data() {
        let input = r"\s:2573598,c:1720090996*00\";
        assert_tag_block(
            input,
            Some(1720090996),
            Some("2573598".to_string()),
            None,
            None,
            None,
            None,
            0x00,
        );
    }

    #[test]
    fn parse_tag_block_with_all_fields() {
        let input = r"\s:2573135,c:1671620143,d:FooBar,n:123,r:456,t:HelloWorld!*36\";
        assert_tag_block(
            input,
            Some(1671620143),
            Some("2573135".to_string()),
            Some("FooBar".to_string()),
            Some(123),
            Some(456),
            Some("HelloWorld!".to_string()),
            0x36,
        );
    }

    #[test]
    fn parse_tag_block_with_invalid_format() {
        let input = r"invalid_tag_block_format";
        let result = TagBlock::parse(input);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Invalid tag block format: missing checksum"
        );
    }

    #[test]
    fn parse_tag_block_with_invalid_checksum_format() {
        let input = r"\s:2573135,c:1671620143*GG\";
        let result = TagBlock::parse(input);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Invalid checksum format: checksum is not a valid hexadecimal number"
        );
    }

    #[test]
    fn parse_tag_block_with_checksum_mismatch() {
        let input = r"\s:2573135,c:1671620143*FF\";
        let result = TagBlock::parse(input);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Checksum mismatch"));
    }

    #[test]
    fn parse_tag_block_with_unknown_keys() {
        let input = r"\x:unknown_key,s:2573135,c:1671620143*25\";
        assert_tag_block(
            input,
            Some(1671620143),
            Some("2573135".to_string()),
            None,
            None,
            None,
            None,
            0x25,
        );
    }
}
