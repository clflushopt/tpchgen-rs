/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Output utilities for TPC-DS data generation
//!
//! The Java implementation reads distribution files as ISO-8859-1 (Latin-1) and
//! writes output files as ISO-8859-1 (see TableGenerator.java line 80).
//!
//! Rust reads ISO-8859-1 bytes and converts them to UTF-8 strings (since Rust
//! strings are UTF-8). For byte-for-byte compatibility with Java output, we must
//! convert back to ISO-8859-1 when writing.
//!
//! Since ISO-8859-1 bytes 0x00-0xFF map directly to Unicode code points U+0000-U+00FF,
//! any character from the distribution files can be safely converted back to a single byte.

use std::io::{self, Write};

/// A wrapper that implements std::io::Write by appending to a String.
///
/// This allows using `write!` macro with a String buffer, which can then
/// be converted to ISO-8859-1 and written to the output.
pub struct StringWriter<'a>(pub &'a mut String);

impl<'a> Write for StringWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // buf contains UTF-8 bytes from write! macro
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.0.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Converts a UTF-8 string to ISO-8859-1 bytes.
///
/// This is the inverse of the conversion done in file_loader.rs when reading
/// distribution files. Characters must be in the range U+0000-U+00FF.
///
/// # Errors
/// Returns an error if any character is outside the ISO-8859-1 range (U+0000-U+00FF).
pub fn to_iso_8859_1(s: &str) -> io::Result<Vec<u8>> {
    s.chars()
        .map(|c| {
            let code = c as u32;
            if code > 255 {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Character '{}' (U+{:04X}) is outside ISO-8859-1 range",
                        c, code
                    ),
                ))
            } else {
                Ok(code as u8)
            }
        })
        .collect()
}

/// A writer wrapper that converts UTF-8 strings to ISO-8859-1 before writing.
///
/// This matches Java's behavior in TableGenerator.java which writes output
/// using StandardCharsets.ISO_8859_1.
pub struct Iso8859Writer<W: Write> {
    inner: W,
}

impl<W: Write> Iso8859Writer<W> {
    pub fn new(writer: W) -> Self {
        Iso8859Writer { inner: writer }
    }

    /// Write a string as ISO-8859-1 bytes
    pub fn write_str(&mut self, s: &str) -> io::Result<()> {
        let bytes = to_iso_8859_1(s)?;
        self.inner.write_all(&bytes)
    }

    /// Write a string followed by a newline as ISO-8859-1 bytes
    pub fn write_line(&mut self, s: &str) -> io::Result<()> {
        self.write_str(s)?;
        self.inner.write_all(b"\n")
    }

    /// Flush the underlying writer
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Implement std::io::Write for Iso8859Writer so it can be used with write! macro
/// and TableRow::write_to().
///
/// The input bytes are expected to be valid UTF-8 (as produced by write! macro).
/// Each UTF-8 character is converted to its ISO-8859-1 equivalent.
impl<W: Write> Write for Iso8859Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Interpret input as UTF-8, convert to ISO-8859-1
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let iso_bytes = to_iso_8859_1(s)?;
        self.inner.write_all(&iso_bytes)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_iso_8859_1_ascii() {
        let result = to_iso_8859_1("Hello").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_to_iso_8859_1_latin1() {
        // Ô is U+00D4, which should become byte 0xD4
        let result = to_iso_8859_1("CÔTE D'IVOIRE").unwrap();
        assert_eq!(result[1], 0xD4); // The Ô character
        assert_eq!(result.len(), 13); // One byte per character
    }

    #[test]
    fn test_iso8859_writer() {
        let mut buffer = Vec::new();
        {
            let mut writer = Iso8859Writer::new(&mut buffer);
            writer.write_line("CÔTE D'IVOIRE").unwrap();
        }
        // Verify Ô (U+00D4) is written as single byte 0xD4, not UTF-8 (0xC3 0x94)
        assert_eq!(buffer[1], 0xD4);
        assert_eq!(buffer.len(), 14); // 13 chars + newline
    }

    #[test]
    fn test_to_iso_8859_1_out_of_range() {
        // Euro sign € is U+20AC, outside ISO-8859-1 range
        let result = to_iso_8859_1("€100");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("outside ISO-8859-1 range"));
    }

    #[test]
    fn test_string_writer() {
        let mut buffer = String::new();
        {
            let mut writer = StringWriter(&mut buffer);
            write!(writer, "Hello, {}!", "World").unwrap();
        }
        assert_eq!(buffer, "Hello, World!");
    }

    #[test]
    fn test_string_writer_with_numbers() {
        let mut buffer = String::new();
        {
            let mut writer = StringWriter(&mut buffer);
            write!(writer, "{}|{}|{}", 42, "test", 3.14).unwrap();
        }
        assert_eq!(buffer, "42|test|3.14");
    }
}
