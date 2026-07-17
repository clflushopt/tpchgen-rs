use crate::row::table_row::DatField;
use crate::row::TableRow;
use std::fmt;

/// Represents a row in the TIME_DIM table
#[derive(Debug, Clone)]
pub struct TimeDimRow {
    // Null bitmap for handling NULL values
    null_bit_map: i64,

    // Primary key (seconds since midnight)
    pub t_time_sk: i64,

    // Time identifier (HHMMSS format)
    pub t_time_id: String,

    // Time value (seconds since midnight)
    pub t_time: i32,

    // Time components
    pub t_hour: i32,
    pub t_minute: i32,
    pub t_second: i32,

    // AM/PM indicator
    pub t_am_pm: String,

    // Shift information
    pub t_shift: String,
    pub t_sub_shift: String,

    // Meal time classification
    pub t_meal_time: String,
}

impl TimeDimRow {
    #[allow(clippy::too_many_arguments)]
    /// Create a new TimeDimRow with all fields
    pub fn new(
        null_bit_map: i64,
        t_time_sk: i64,
        t_time_id: String,
        t_time: i32,
        t_hour: i32,
        t_minute: i32,
        t_second: i32,
        t_am_pm: String,
        t_shift: String,
        t_sub_shift: String,
        t_meal_time: String,
    ) -> Self {
        TimeDimRow {
            null_bit_map,
            t_time_sk,
            t_time_id,
            t_time,
            t_hour,
            t_minute,
            t_second,
            t_am_pm,
            t_shift,
            t_sub_shift,
            t_meal_time,
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    /// Check if a column should be NULL based on the null bitmap
    fn is_field_null(&self, column_index: usize) -> bool {
        (self.null_bit_map & (1 << column_index)) != 0
    }

    /// Get string value or NULL for optional fields
    fn get_string_or_null<T: ToString>(&self, value: T, column_index: usize) -> String {
        if self.is_field_null(column_index) {
            String::new()
        } else {
            value.to_string()
        }
    }
}

/// DAT field helper mirroring `get_string_or_null`.
impl TimeDimRow {
    fn field<T>(&self, value: T, column_index: usize) -> DatField<T> {
        DatField::new(value, self.is_field_null(column_index))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for TimeDimRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.t_time_sk, 0),
            self.field(&self.t_time_id, 1),
            self.field(self.t_time, 2),
            self.field(self.t_hour, 3),
            self.field(self.t_minute, 4),
            self.field(self.t_second, 5),
            self.field(&self.t_am_pm, 6),
            self.field(&self.t_shift, 7),
            self.field(&self.t_sub_shift, 8),
            self.field(&self.t_meal_time, 9),
        )
    }
}

impl TableRow for TimeDimRow {
    fn get_values(&self) -> Vec<String> {
        vec![
            self.get_string_or_null(self.t_time_sk, 0),
            self.get_string_or_null(&self.t_time_id, 1),
            self.get_string_or_null(self.t_time, 2),
            self.get_string_or_null(self.t_hour, 3),
            self.get_string_or_null(self.t_minute, 4),
            self.get_string_or_null(self.t_second, 5),
            self.get_string_or_null(&self.t_am_pm, 6),
            self.get_string_or_null(&self.t_shift, 7),
            self.get_string_or_null(&self.t_sub_shift, 8),
            self.get_string_or_null(&self.t_meal_time, 9),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        let row = TimeDimRow::new(
            0b10,
            36000,
            "AAAAAAAABKAAAAAA".to_string(),
            36000,
            10,
            0,
            0,
            "AM".to_string(),
            "morning".to_string(),
            "breakfast".to_string(),
            "".to_string(),
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
