use std::fmt;

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

impl<T> DatField<T> {
    /// DAT field for a regular value: NULL when the row's null bit is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        DatField((!is_null).then_some(value))
    }
}

/// DAT field from an already-computed optional value, for callers whose
/// value is only constructible when non-NULL.
impl<T> From<Option<T>> for DatField<T> {
    fn from(value: Option<T>) -> Self {
        DatField(value)
    }
}

impl DatField<i64> {
    /// DAT field for a surrogate key: NULL when the row's null bit is set or
    /// the key is -1 (the generators' "no reference" sentinel).
    pub(crate) fn key(key: i64, is_null: bool) -> Self {
        DatField((!is_null && key != -1).then_some(key))
    }
}

impl DatField<&'static str> {
    /// DAT field for a boolean, formatted as `Y`/`N`.
    pub(crate) fn yes_no(value: bool, is_null: bool) -> Self {
        DatField::new(if value { "Y" } else { "N" }, is_null)
    }
}

/// A DAT field that prints the literal `NULL` for NULL columns instead of an
/// empty string — a quirk of the Java call_center and household_demographics
/// rows that we preserve for byte-for-byte compatibility.
pub(crate) struct NullLiteralField<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for NullLiteralField<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => value.fmt(f),
            None => f.write_str("NULL"),
        }
    }
}

impl<T> NullLiteralField<T> {
    /// DAT field printing the literal `NULL` when the row's null bit is set.
    pub(crate) fn new(value: T, is_null: bool) -> Self {
        NullLiteralField((!is_null).then_some(value))
    }
}

/// Zero-padded five-digit zip code (`{:05}`).
pub(crate) struct Zip5(i32);

impl fmt::Display for Zip5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:05}", self.0)
    }
}

impl DatField<Zip5> {
    /// DAT field for a zip code, zero-padded to five digits.
    pub(crate) fn zip(zip: i32, is_null: bool) -> Self {
        DatField::new(Zip5(zip), is_null)
    }
}

/// TableRow trait matching the Java TableRow interface
/// Represents a single row of data from any TPC-DS table
pub trait TableRow: Send + Sync {
    /// Get all values as strings for output (getValues())
    ///
    /// Note: This method allocates a `Vec<String>`. For performance-critical code,
    /// prefer using the row's `fmt::Display` impl which formats directly into a
    /// buffer.
    fn get_values(&self) -> Vec<String>;

    /// Get the number of columns in this row
    fn get_column_count(&self) -> usize {
        self.get_values().len()
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
}
