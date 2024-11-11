//! Standard Class B Position Report (type 18)
use super::parsers::*;
use super::AisMessageType;
use crate::errors::Result;
use crate::lib;
use nom::bits::{bits, complete::take as take_bits};
use nom::combinator::map;
use nom::IResult;
use serde::{Deserialize, Serialize};

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
const MAX_DATA_SIZE_BYTES: usize = 119;

#[cfg(any(feature = "std", feature = "alloc"))]
pub type CorrectionData = lib::std::vec::Vec<u8>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type CorrectionData = lib::std::vec::Vec<u8, MAX_DATA_SIZE_BYTES>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DgnssBroadcastBinaryMessage {
    pub message_type: u8,
    pub repeat_indicator: u8,
    pub mmsi: u32,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub payload: DifferentialCorrectionData,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifferentialCorrectionData {
    pub message_type: u8,
    pub station_id: u16,
    pub z_count: u16,
    pub sequence_number: u8,
    pub n: u8,
    pub health: u8,
    pub data: CorrectionData,
}

impl DifferentialCorrectionData {
    fn parse(data: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, station_id) = take_bits(10u8)(data)?;
        let (data, z_count) = take_bits(13u8)(data)?;
        let (data, sequence_number) = take_bits(3u8)(data)?;
        let (data, n) = take_bits(5u8)(data)?;
        let (data, health) = take_bits(3u8)(data)?;
        #[cfg(any(feature = "std", feature = "alloc"))]
        let data_owned = data.0.into();
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let data_owned = data.0.try_into().map_err(|_| {
            nom::Err::Failure(nom::error::Error::new(
                data,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;
        Ok((
            (<&[u8]>::default(), 0),
            Self {
                message_type,
                station_id,
                z_count,
                sequence_number,
                n,
                health,
                data: data_owned,
            },
        ))
    }
}

impl<'a> AisMessageType<'a> for DgnssBroadcastBinaryMessage {
    fn name(&self) -> &'static str {
        "DGNSS Broadcast Binary Message"
    }

    fn parse(data: &'a [u8]) -> Result<Self> {
        let (_, message) = parse_base(data)?;
        Ok(message)
    }
}

fn parse_longitude_min_10(data: i32) -> Option<f32> {
    match data {
        108_600 => None,
        _ => Some(data as f32 / 600.0),
    }
}

fn parse_latitude_min_10(data: i32) -> Option<f32> {
    match data {
        54_600 => None,
        _ => Some(data as f32 / 600.0),
    }
}

fn parse_base(data: &[u8]) -> IResult<&[u8], DgnssBroadcastBinaryMessage> {
    bits(move |data| -> IResult<_, _> {
        let (data, message_type) = take_bits(6u8)(data)?;
        let (data, repeat_indicator) = take_bits(2u8)(data)?;
        let (data, mmsi) = take_bits(30u32)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(2u8)(data)?;
        let (data, longitude) = map(|data| signed_i32(data, 18), parse_longitude_min_10)(data)?;
        let (data, latitude) = map(|data| signed_i32(data, 17), parse_latitude_min_10)(data)?;
        let (data, _spare) = take_bits::<_, u8, _, _>(5u8)(data)?;
        let (data, payload) = DifferentialCorrectionData::parse(data)?;
        Ok((
            data,
            DgnssBroadcastBinaryMessage {
                message_type,
                repeat_indicator,
                mmsi,
                longitude,
                latitude,
                payload,
            },
        ))
    })(data)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_message() {
        let bytestream =
            b"A02VqLPA4I6C07h5Ed1h<OrsuBTTwS?r:C?w`?la<gno1RTRwSP9:BcurA8a:Oko02TSwu8<:Jbb";
        let bitstream = crate::messages::unarmor(bytestream, 0).unwrap();
        let message = DgnssBroadcastBinaryMessage::parse(bitstream.as_ref()).unwrap();
        assert_eq!(message.message_type, 17);
        assert_eq!(message.repeat_indicator, 0);
        assert_eq!(message.mmsi, 2734450);
        f32_equal_naive(message.longitude.unwrap(), 29.13);
        f32_equal_naive(message.latitude.unwrap(), 59.986668);
        assert_eq!(message.payload.z_count, 2776);
        assert_eq!(message.payload.data.len(), 42);
    }
}
