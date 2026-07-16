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

use crate::row::table_row::DatField;
use crate::row::TableRow;
use crate::types::Date;
use std::fmt;

/// DbgenVersion table row
#[derive(Debug, Clone)]
pub struct DbgenVersionRow {
    null_bit_map: i64,
    pub(crate) dv_version: String,
    pub(crate) dv_create_date: Date,
    pub(crate) dv_create_time: i32,
    pub(crate) dv_cmdline_args: String,
}

impl DbgenVersionRow {
    pub fn new(
        null_bit_map: i64,
        dv_version: String,
        dv_create_date: Date,
        dv_create_time: i32,
        dv_cmdline_args: String,
    ) -> Self {
        DbgenVersionRow {
            null_bit_map,
            dv_version,
            dv_create_date,
            dv_create_time,
            dv_cmdline_args,
        }
    }

    /// Check if a column should be null based on the null bitmap
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    /// Convert value to string or empty string if null
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

    pub fn get_dv_version(&self) -> &str {
        &self.dv_version
    }

    pub fn get_dv_create_date(&self) -> &Date {
        &self.dv_create_date
    }

    pub fn get_dv_create_time(&self) -> i32 {
        self.dv_create_time
    }

    pub fn get_dv_cmdline_args(&self) -> &str {
        &self.dv_cmdline_args
    }
}

/// Seconds-since-midnight rendered as `HH:MM:SS`, like `format_time`.
pub(crate) struct TimeOfDay(pub(crate) i32);

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hour = self.0 / 3600;
        let minute = (self.0 / 60) % 60;
        let second = self.0 % 60;
        write!(f, "{hour:02}:{minute:02}:{second:02}")
    }
}

/// DAT field helper mirroring `get_string_or_null`.
impl DbgenVersionRow {
    pub(crate) fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for DbgenVersionRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|",
            self.field(&self.dv_version, 0),
            self.field(self.dv_create_date, 1),
            self.field(TimeOfDay(self.dv_create_time), 2),
            self.field(&self.dv_cmdline_args, 3),
        )
    }
}

impl TableRow for DbgenVersionRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java DbgenVersionGeneratorColumn (476-479)
        vec![
            self.get_string_or_null(&self.dv_version, 0),
            self.get_string_or_null(self.dv_create_date, 1),
            self.get_string_or_null(format_time(self.dv_create_time), 2),
            self.get_string_or_null(&self.dv_cmdline_args, 3),
        ]
    }
}

fn format_time(seconds_since_midnight: i32) -> String {
    let hour = seconds_since_midnight / 3600;
    let minute = (seconds_since_midnight / 60) % 60;
    let second = seconds_since_midnight % 60;
    format!("{hour:02}:{minute:02}:{second:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        let row = DbgenVersionRow::new(
            0b10,
            "2.0.0".to_string(),
            Date::from_julian_days(2452539),
            37231,
            "-SCALE 1".to_string(),
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
