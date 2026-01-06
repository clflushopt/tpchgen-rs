use std::io::{self, Write};

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
