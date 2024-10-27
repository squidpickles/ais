# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2024-10-07
### Added
- Support for message type 6 (Binary Addressed Message) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 7 (Binary Acknowledge) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 9 (Standard SAR Aircraft Position Report) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 10 (UTC/Date Inquiry) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 12 (Addressed Safety-Related Message) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 13 (Safety-Related Acknowledgment) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 14 (Safety-Related Broadcast Message) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 16 (Assignment Mode Command) (thanks [@salsabiljb](https://github.com/salsabiljb))
- Support for message type 27 (Long-Range AIS Broadcast Message) (thanks [@salsabiljb](https://github.com/salsabiljb))

## [0.11.0] - 2023-11-05
### Added
- NMEA tag blocks are now accepted, but currently ignored (thanks [@jkr78](https://github.com/jkr78))
### Changed
- `radio_status` and `navigation` modules are now `pub` (thanks [@ebuckley](https://github.com/ebuckley))

## [0.10.0] - 2023-02-17
### Added
- `From<u8> for ShipType` and `From<ShipType> for u8` implementations (thanks [@keesverruijt](https://github.com/keesverruijt))
### Changed
- `types` module is now `pub` (thanks [@keesverruijt](https://github.com/keesverruijt))
- `ShipType` reserved fields are included as data in the enum (thanks [@keesverruijt](https://github.com/keesverruijt))

## [0.9.0] - 2022-09-17
### Added
- Support for message type 11 (UTC/Date Response)
- `no_std` support

## [0.8.0] - 2021-11-13
### Added
- Support for message type 19 (Extended Class B Position Report)
### Changed
- Edition 2021
- Updated Nom dependency to v7
- `AssignedMode` moved into `messages.types`

## [0.7.0] - 2021-03-11
### Added
- Support for message type 17 (DGNSS Broadcast Binary Message).

## [0.6.0] - 2020-05-02
### Added
- Support for message type 8 (Binary Broadcast Message). Note, the binary payload is not decoded yet.
- Support for message type 20 (Data Link Management Message)
- Support for message type 15 (Interrogation)

## [0.5.0] - 2020-05-01
### Added
- Support for message type 18 (Static Class B Position Report)
### Changed
- Utility renamed from `nmea` to `ais`

## [0.4.0] - 2020-04-28
### Changed
- Many internal types with restricted inputs panic rather than returning errors
- Out-of-range values coming from parsed data now get passed through as unknown, rather than returning errors
- UTC hour is no longer Option

## [0.3.0] - 2020-04-28
### Added
- Support for type 24 messages
- Export `AisParser` and `AisFragments` at crate level

### Changed
- Channel returns an `Option` to support missing channels (seen in real-world)
- Maneuverability and date indices out of spec no longer throw an error

## [0.2.0] - 2020-04-26
### Added
- Support for type 5 messages
- Support for fragmented sentences

### Changed
- Top level interface now involves using an `AisParser` object
- Message parsing happens at the same time as NMEA sentence parsing, if enabled
- Updated Nom dependency to v5
- Replaced error-chain with thiserror

## [0.1.1] - 2020-04-18
### Added
- Link to documentation at https://docs.rs/ais

## [0.1.0] - 2020-04-18
### Added
- Initial release
