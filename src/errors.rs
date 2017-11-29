error_chain!{
    errors {
        Nmea(msg: String) {
            description("invalid NMEA message")
            display("invalid NMEA message: '{}'", msg)
        }
        Checksum(expected: u8, received: u8) {
            description("checksum mismatch")
            display("checksum mismatch; expected: {:#X}, received: {:#X}", expected, received)
        }
    }
}
