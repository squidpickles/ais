use crate::errors::Result;
use crate::messages::AisMessage;
use crate::AisParser;

/// An iterator adapter that wraps an iterator of NMEA sentence lines and yields
/// parsed [`AisMessage`]s. Incomplete (multi-fragment) sentences are accumulated
/// internally until a complete message is ready.
///
/// Created by [`AisParser::iter()`].
pub struct AisMessageIterator<I> {
    inner: I,
    parser: AisParser,
}

impl<I, T> Iterator for AisMessageIterator<I>
where
    I: Iterator<Item = T>,
    T: AsRef<[u8]>,
{
    type Item = Result<AisMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.inner.next()?;
            match self.parser.parse_message(line.as_ref()) {
                Ok(Some(msg)) => return Some(Ok(msg)),
                Ok(None) => continue,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

#[cfg(feature = "std")]
use crate::errors::Error;
#[cfg(feature = "std")]
use std::io::BufRead;

/// An iterator that reads NMEA sentences from a [`BufRead`] source and yields
/// parsed [`AisMessage`]s. Lines are read one at a time; incomplete fragments
/// are accumulated until a complete message is available.
///
/// Created by [`AisParser::read_messages()`].
#[cfg(feature = "std")]
pub struct AisMessageReader<R> {
    inner: R,
    parser: AisParser,
    buf: String,
}

#[cfg(feature = "std")]
impl<R: BufRead> Iterator for AisMessageReader<R> {
    type Item = Result<AisMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.inner.read_line(&mut self.buf) {
                Ok(0) => return None,
                Ok(_) => {
                    let line = self.buf.trim_end();
                    if line.is_empty() {
                        continue;
                    }
                    match self.parser.parse_message(line.as_bytes()) {
                        Ok(Some(msg)) => return Some(Ok(msg)),
                        Ok(None) => continue,
                        Err(e) => return Some(Err(e)),
                    }
                }
                Err(e) => return Some(Err(Error::Nmea { msg: e.to_string() })),
            }
        }
    }
}

impl AisParser {
    /// Creates an iterator that parses AIS messages from an iterator of NMEA
    /// sentence lines. Each item should be a single NMEA sentence as bytes.
    ///
    /// Multi-fragment messages are assembled automatically. The iterator yields
    /// one [`AisMessage`] per complete message, or an error if parsing fails.
    ///
    /// # Example:
    /// ```
    /// use ais::AisParser;
    ///
    /// let lines = vec![
    ///     b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01".to_vec(),
    /// ];
    /// for result in AisParser::iter_messages(lines) {
    ///     match result {
    ///         Ok(message) => { /* process message */ },
    ///         Err(e) => { /* handle error */ },
    ///     }
    /// }
    /// ```
    pub fn iter_messages<I, T>(lines: I) -> AisMessageIterator<I::IntoIter>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        AisMessageIterator {
            inner: lines.into_iter(),
            parser: AisParser::new(),
        }
    }

    /// Creates an iterator that reads NMEA sentences from a [`BufRead`] source
    /// and yields parsed [`AisMessage`]s.
    ///
    /// # Example:
    /// ```
    /// use std::io::Cursor;
    /// use ais::AisParser;
    ///
    /// let data = b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01\n";
    /// let reader = Cursor::new(&data[..]);
    /// for result in AisParser::read_messages(reader) {
    ///     match result {
    ///         Ok(message) => { /* process message */ },
    ///         Err(e) => { /* handle error */ },
    ///     }
    /// }
    /// ```
    #[cfg(feature = "std")]
    pub fn read_messages<R: BufRead>(reader: R) -> AisMessageReader<R> {
        AisMessageReader {
            inner: reader,
            parser: AisParser::new(),
            buf: String::new(),
        }
    }
}
