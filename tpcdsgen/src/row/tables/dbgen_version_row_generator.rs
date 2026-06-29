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
use chrono::Local;

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
        // Get current date and time
        let now = Local::now();

        // Format date as "yyyy-MM-dd" (Java SimpleDateFormat equivalent)
        let create_date = now.format("%Y-%m-%d").to_string();

        // Format time as "HH:mm:ss" (Java SimpleDateFormat equivalent)
        let create_time = now.format("%H:%M:%S").to_string();

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
