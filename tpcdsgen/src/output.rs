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
//! The Java (Trino) implementation reads distribution files as ISO-8859-1
//! (Latin-1) and writes output files as ISO-8859-1 (see TableGenerator.java
//! line 80). The C `dsdgen` outputs UTF-8.
//!
//! [`CompatWriter`] selects the right behavior based on [`CompatMode`].

use std::fmt;
use std::io::{self, Write};

use crate::config::CompatMode;

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
/// This matches Trino's behavior in TableGenerator.java which writes output
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

/// Writer that selects the output encoding based on [`CompatMode`].
///
/// * `Iso8859`: outputs ISO-8859-1 to match Trino.
/// * `Utf8`: outputs UTF-8 to match unmodified C `dsdgen`.
pub enum CompatWriter<W: Write> {
    Iso8859(Iso8859Writer<W>),
    Utf8(W),
}

impl<W: Write> CompatWriter<W> {
    /// Build a writer for `compat_mode`.
    pub fn new(writer: W, compat_mode: CompatMode) -> Self {
        match compat_mode {
            CompatMode::Trino => CompatWriter::Iso8859(Iso8859Writer::new(writer)),
            CompatMode::C => CompatWriter::Utf8(writer),
        }
    }
}

impl<W: Write> Write for CompatWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            CompatWriter::Iso8859(w) => w.write(buf),
            CompatWriter::Utf8(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            CompatWriter::Iso8859(w) => w.flush(),
            CompatWriter::Utf8(w) => w.flush(),
        }
    }
}

/// Buffered DAT row writer.
///
/// Rows are formatted into an in-memory UTF-8 buffer which is encoded and
/// flushed to the inner writer one large chunk at a time. Encoding per chunk
/// instead of per field keeps the hot path to plain byte appends: the common
/// all-ASCII chunk is written through unchanged in both compat modes (ASCII
/// bytes are identical in UTF-8 and ISO-8859-1), and only chunks that
/// actually contain non-ASCII characters pay for ISO-8859-1 conversion in
/// Trino mode.
pub struct DatWriter<W: Write> {
    inner: W,
    compat_mode: CompatMode,
    /// Pending formatted rows (UTF-8).
    buffer: Vec<u8>,
}

impl<W: Write> DatWriter<W> {
    /// Flush the pending buffer once it grows past this size: large enough to
    /// amortize the encoding scan and write call, small enough to stay in L2.
    const FLUSH_THRESHOLD: usize = 64 * 1024;

    pub fn new(inner: W, compat_mode: CompatMode) -> Self {
        Self {
            inner,
            compat_mode,
            buffer: Vec::with_capacity(Self::FLUSH_THRESHOLD + 1024),
        }
    }

    /// Write one row whose `Display` impl emits the DAT line (fields joined
    /// by the separator, with a trailing separator); appends the newline.
    pub fn write_display_row(&mut self, row: &impl fmt::Display) -> io::Result<()> {
        writeln!(self.buffer, "{row}")?;
        self.maybe_flush()
    }

    /// Encode and flush the pending buffer if it has grown past the threshold.
    fn maybe_flush(&mut self) -> io::Result<()> {
        if self.buffer.len() >= Self::FLUSH_THRESHOLD {
            self.flush_buffer()?;
        }
        Ok(())
    }

    fn flush_buffer(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        match self.compat_mode {
            CompatMode::C => self.inner.write_all(&self.buffer)?,
            CompatMode::Trino => {
                if self.buffer.is_ascii() {
                    self.inner.write_all(&self.buffer)?;
                } else {
                    let s = std::str::from_utf8(&self.buffer)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    self.inner.write_all(&to_iso_8859_1(s)?)?;
                }
            }
        }
        self.buffer.clear();
        Ok(())
    }

    /// Flush all pending rows and the inner writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer()?;
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
    fn test_compat_writer_trino_emits_iso_8859_1() {
        let mut buffer = Vec::new();
        {
            let mut writer = CompatWriter::new(&mut buffer, CompatMode::Trino);
            write!(writer, "CÔTE D'IVOIRE").unwrap();
        }
        // Trino/Java emits a single 0xD4 byte for Ô.
        assert_eq!(buffer[1], 0xD4);
        assert_eq!(buffer.len(), 13);
    }

    struct TestRow;

    impl std::fmt::Display for TestRow {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "1|CÔTE|2.50|")
        }
    }

    #[test]
    fn test_dat_writer_buffers_until_flush() {
        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::Trino);
        writer.write_display_row(&TestRow).unwrap();
        // Small rows stay buffered until an explicit flush.
        assert!(!writer.buffer.is_empty());
        writer.flush().unwrap();
        // Ô (U+00D4) must come out as the single ISO-8859-1 byte 0xD4.
        assert_eq!(out, b"1|C\xD4TE|2.50|\n");
    }

    #[test]
    fn test_dat_writer_utf8_mode_passes_through() {
        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::C);
        writer.write_display_row(&TestRow).unwrap();
        writer.flush().unwrap();
        assert_eq!(out, "1|CÔTE|2.50|\n".as_bytes());
    }

    #[test]
    fn test_dat_writer_rejects_non_latin1_in_trino_mode() {
        struct EuroRow;
        impl std::fmt::Display for EuroRow {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "€100|")
            }
        }

        let mut out = Vec::new();
        let mut writer = DatWriter::new(&mut out, CompatMode::Trino);
        writer.write_display_row(&EuroRow).unwrap();
        let err = writer.flush().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_compat_writer_c_emits_utf8() {
        let mut buffer = Vec::new();
        {
            let mut writer = CompatWriter::new(&mut buffer, CompatMode::C);
            write!(writer, "CÔTE D'IVOIRE").unwrap();
        }
        // C dsdgen passes the UTF-8 bytes through (Ô is 0xC3 0x94).
        assert_eq!(&buffer[..3], &[b'C', 0xC3, 0x94]);
        assert_eq!(buffer.len(), 14);
    }
}
