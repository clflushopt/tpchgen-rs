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

use crate::row::TableRow;

/// DbgenVersion table row
#[derive(Debug, Clone)]
pub struct DbgenVersionRow {
    null_bit_map: i64,
    dv_version: String,
    dv_create_date: String,
    dv_create_time: String,
    dv_cmdline_args: String,
}

impl DbgenVersionRow {
    pub fn new(
        null_bit_map: i64,
        dv_version: String,
        dv_create_date: String,
        dv_create_time: String,
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

    pub fn get_dv_version(&self) -> &str {
        &self.dv_version
    }

    pub fn get_dv_create_date(&self) -> &str {
        &self.dv_create_date
    }

    pub fn get_dv_create_time(&self) -> &str {
        &self.dv_create_time
    }

    pub fn get_dv_cmdline_args(&self) -> &str {
        &self.dv_cmdline_args
    }
}

impl TableRow for DbgenVersionRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java DbgenVersionGeneratorColumn (476-479)
        vec![
            self.get_string_or_null(&self.dv_version, 0),
            self.get_string_or_null(&self.dv_create_date, 1),
            self.get_string_or_null(&self.dv_create_time, 2),
            self.get_string_or_null(&self.dv_cmdline_args, 3),
        ]
    }
}
