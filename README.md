# AIS parser

This is the beginning of a parser for [AIS](https://en.wikipedia.org/wiki/Automatic_identification_system) messages, written in Rust.

## Try it out
If you just want to take a bunch of AIS NMEA sentences and see what they mean, you can try running this as a (very rough) command-line utility.

If your NMEA source is sending UDP packets to port 4722, for example:

```bash
nc -u -l 4722 | cargo run
```

You should start seeing messages stream in:
```
AidToNavigationReport(AidToNavigationReport { message_type: 21, repeat_indicator: 1, mmsi: 993692016, aid_type: Some(ReferencePoint), name: "6W", accuracy: Unaugmented, longitude: Some(-122.80445), latitude: Some(37.705833), dimension_to_bow: 0, dimension_to_stern: 0, dimension_to_port: 0, dimension_to_starboard: 0, epfd_type: Some(Surveyed), utc_second: 61, off_position: false, regional_reserved: 0, raim: false, virtual_aid: false, assigned_mode: false })
BaseStationReport(BaseStationReport { message_type: 4, repeat_indicator: 0, mmsi: 3669710, year: Some(2020), month: Some(4), day: Some(18), hour: Some(8), minute: Some(46), second: Some(40), fix_quality: DGPS, longitude: Some(-122.42347), latitude: Some(37.96206), epfd_type: None, raim: true, radio_status: Sotdma(SotdmaMessage { sync_state: UtcDirect, slot_timeout: 0, sub_message: SlotOffset(2250) }) })
PositionReport(PositionReport { message_type: 1, repeat_indicator: 0, mmsi: 367625810, navigation_status: Some(UnderWayUsingEngine), rate_of_turn: Some(RateOfTurn { raw: 0 }), speed_over_ground: Some(0.1), position_accuracy: DGPS, longitude: Some(-122.398), latitude: Some(37.80256), course_over_ground: Some(343.8), true_heading: Some(55), timestamp: 41, maneuver_indicator: None, raim: false, radio_status: Sotdma(SotdmaMessage { sync_state: UtcDirect, slot_timeout: 2, sub_message: SlotNumber(1524) }) })
BaseStationReport(BaseStationReport { message_type: 4, repeat_indicator: 0, mmsi: 3669145, year: Some(2020), month: Some(4), day: Some(18), hour: Some(8), minute: Some(46), second: Some(41), fix_quality: DGPS, longitude: Some(-122.46484), latitude: Some(37.794273), epfd_type: None, raim: true, radio_status: Sotdma(SotdmaMessage { sync_state: UtcDirect, slot_timeout: 3, sub_message: ReceivedStations(187) }) })
```

## Use it as a library
Here's a very simple example that parses a single NMEA sentence. In this case, it contains an Aid to Navigation Report:
 ```rust
use ais::sentence::AisSentence;
use ais::messages::AisMessage;

// The line below is an NMEA sentence, much as you'd see coming out of an AIS decoder.
let line = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01";

let sentence = AisSentence::parse(line)?;
// This sentence is complete, ie unfragmented
assert_eq!(sentence.num_fragments, 1);
// The data was transmitted on AIS channel B
assert_eq!(sentence.channel, 'B');

// Now we parse the message itself
match sentence.message()? {
    AisMessage::AidToNavigationReport(report) => {
        assert_eq!(report.mmsi, 993692028);
        assert_eq!(report.name, "SF OAK BAY BR VAIS E");
        // There are a ton more fields available here
    },
    _ => panic!("Unexpected message type"),
}
# Ok::<(), ais::errors::Error>(())
```

## Supported message types
Right now, only a few common types are supported. They are:

- Position Report (types 1-3)
- Base Station Report (type 4)
- Aid to Navigation Report (type 21)

Others to come soon, I hope!
