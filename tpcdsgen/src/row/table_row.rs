use std::fmt;
use std::io::{self, Write};

/// A single DAT-format field: formats the value, or nothing when the column
/// is NULL.
///
/// Row types use this in their `Display` impls (which emit the DAT line) so
/// that NULL handling stays out of the format string.
pub(crate) struct DatField<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for DatField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => Ok(()),
        }
    }
}

/// DAT field for a regular value: NULL when the row's null bit is set.
pub(crate) fn dat_field<T>(value: T, is_null: bool) -> DatField<T> {
    DatField((!is_null).then_some(value))
}

/// DAT field for a surrogate key: NULL when the row's null bit is set or the
/// key is -1 (the generators' "no reference" sentinel).
pub(crate) fn dat_key(key: i64, is_null: bool) -> DatField<i64> {
    DatField((!is_null && key != -1).then_some(key))
}

/// TableRow trait matching the Java TableRow interface
/// Represents a single row of data from any TPC-DS table
pub trait TableRow: Send + Sync {
    /// Get all values as strings for output (getValues())
    ///
    /// Note: This method allocates a `Vec<String>`. For performance-critical code,
    /// prefer using `write_to()` which writes directly to a buffer.
    fn get_values(&self) -> Vec<String>;

    /// Get the number of columns in this row
    fn get_column_count(&self) -> usize {
        self.get_values().len()
    }

    /// Write the row directly to a writer, avoiding intermediate allocations.
    ///
    /// Each column value is separated by `separator`, and the row ends with
    /// a trailing separator followed by a newline.
    ///
    /// Default implementation calls `get_values()` - override for better performance.
    ///
    /// Note: Uses `dyn Write` for trait object compatibility. The dynamic dispatch
    /// overhead is negligible compared to I/O costs.
    fn write_to(&self, writer: &mut dyn Write, separator: char) -> io::Result<()> {
        let values = self.get_values();
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                write!(writer, "{}", separator)?;
            }
            write!(writer, "{}", value)?;
        }
        write!(writer, "{}", separator)?;
        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a simple test implementation
    struct TestTableRow {
        values: Vec<String>,
    }

    impl TableRow for TestTableRow {
        fn get_values(&self) -> Vec<String> {
            self.values.clone()
        }
    }

    #[test]
    fn test_table_row_trait() {
        let test_row = TestTableRow {
            values: vec!["1".to_string(), "test".to_string(), "123.45".to_string()],
        };

        let values = test_row.get_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "test");
        assert_eq!(values[2], "123.45");
        assert_eq!(test_row.get_column_count(), 3);
    }

    #[test]
    fn test_write_to() {
        let test_row = TestTableRow {
            values: vec!["1".to_string(), "test".to_string(), "123.45".to_string()],
        };

        let mut buffer = Vec::new();
        test_row.write_to(&mut buffer, '|').unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "1|test|123.45|\n");
    }

    #[test]
    fn test_write_to_empty_values() {
        let test_row = TestTableRow {
            values: vec!["".to_string(), "test".to_string(), "".to_string()],
        };

        let mut buffer = Vec::new();
        test_row.write_to(&mut buffer, '|').unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "|test||\n");
    }
}
