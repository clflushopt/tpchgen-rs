use crate::row::table_row::DatField;
use crate::row::TableRow;
use std::fmt;

/// Income band table row (IncomeBandRow)
#[derive(Debug, Clone)]
pub struct IncomeBandRow {
    null_bit_map: i64,
    pub(crate) ib_income_band_id: i32,
    pub(crate) ib_lower_bound: i32,
    pub(crate) ib_upper_bound: i32,
}

impl IncomeBandRow {
    pub fn new(
        null_bit_map: i64,
        ib_income_band_id: i32,
        ib_lower_bound: i32,
        ib_upper_bound: i32,
    ) -> Self {
        IncomeBandRow {
            null_bit_map,
            ib_income_band_id,
            ib_lower_bound,
            ib_upper_bound,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    /// Convert value to string or empty string if null (getStringOrNull)
    fn get_string_or_null<T: ToString>(&self, value: T, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            value.to_string()
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_ib_income_band_id(&self) -> i32 {
        self.ib_income_band_id
    }

    pub fn get_ib_lower_bound(&self) -> i32 {
        self.ib_lower_bound
    }

    pub fn get_ib_upper_bound(&self) -> i32 {
        self.ib_upper_bound
    }
}

/// DAT field helper mirroring `get_string_or_null`.
impl IncomeBandRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for IncomeBandRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|",
            self.field(self.ib_income_band_id, 0),
            self.field(self.ib_lower_bound, 1),
            self.field(self.ib_upper_bound, 2),
        )
    }
}

impl TableRow for IncomeBandRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java IncomeBandGeneratorColumn
        // First column (IB_INCOME_BAND_ID) is at global position 194, so relative positions are 0-2
        vec![
            self.get_string_or_null(self.ib_income_band_id, 0),
            self.get_string_or_null(self.ib_lower_bound, 1),
            self.get_string_or_null(self.ib_upper_bound, 2),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        let row = IncomeBandRow::new(0b10, 1, 0, 10000);
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
