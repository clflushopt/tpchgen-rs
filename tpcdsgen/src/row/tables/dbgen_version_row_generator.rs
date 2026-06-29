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

use crate::config::Session;
use crate::error::Result;
use crate::row::{AbstractRowGenerator, DbgenVersionRow, RowGenerator, RowGeneratorResult};
use crate::table::Table;
use std::time::{SystemTime, UNIX_EPOCH};

/// Row generator for the DBGEN_VERSION table (DbgenVersionRowGenerator)
pub struct DbgenVersionRowGenerator {
    abstract_generator: AbstractRowGenerator,
}

/// DBGEN_VERSION constant from Java implementation
const DBGEN_VERSION: &str = "2.0.0";

impl Default for DbgenVersionRowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DbgenVersionRowGenerator {
    /// Create a new DbgenVersionRowGenerator
    pub fn new() -> Self {
        Self {
            abstract_generator: AbstractRowGenerator::new(Table::DbgenVersion),
        }
    }

    /// Generate a DbgenVersionRow with current timestamp and version info
    fn generate_dbgen_version_row(
        &mut self,
        _row_number: i64,
        session: &Session,
    ) -> Result<DbgenVersionRow> {
        let (create_date, create_time) = current_utc_date_time();

        // Get command line arguments from session
        let cmdline_args = session.get_command_line_arguments();

        Ok(DbgenVersionRow::new(
            0, // nullBitMap is always 0 for this table
            DBGEN_VERSION.to_string(),
            create_date,
            create_time,
            cmdline_args,
        ))
    }
}

fn current_utc_date_time() -> (String, String) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    (
        format!("{year:04}-{month:02}-{day:02}"),
        format!("{hour:02}:{minute:02}:{second:02}"),
    )
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let days = days_since_unix_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

impl RowGenerator for DbgenVersionRowGenerator {
    fn generate_row_and_child_rows(
        &mut self,
        row_number: i64,
        session: &Session,
        _parent_row_generator: Option<&mut dyn RowGenerator>,
        _child_row_generator: Option<&mut dyn RowGenerator>,
    ) -> Result<RowGeneratorResult> {
        let row = self.generate_dbgen_version_row(row_number, session)?;
        Ok(RowGeneratorResult::new(row))
    }

    fn consume_remaining_seeds_for_row(&mut self) {
        self.abstract_generator.consume_remaining_seeds_for_row();
    }

    fn skip_rows_until_starting_row_number(&mut self, starting_row_number: i64) {
        self.abstract_generator
            .skip_rows_until_starting_row_number(starting_row_number);
    }
}
