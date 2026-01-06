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

//! Catalog page row structure and formatting

use crate::generator::{CatalogPageGeneratorColumn, GeneratorColumn};
use crate::row::TableRow;

/// Catalog page row
#[derive(Clone)]
pub struct CatalogPageRow {
    null_bit_map: i64,
    cp_catalog_page_sk: i64,
    cp_catalog_page_id: String,
    cp_start_date_id: i64,
    cp_end_date_id: i64,
    cp_department: String,
    cp_catalog_number: i32,
    cp_catalog_page_number: i32,
    cp_description: String,
    cp_type: String,
}

impl CatalogPageRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cp_catalog_page_sk: i64,
        cp_catalog_page_id: String,
        cp_start_date_id: i64,
        cp_end_date_id: i64,
        cp_department: String,
        cp_catalog_number: i32,
        cp_catalog_page_number: i32,
        cp_description: String,
        cp_type: String,
    ) -> Self {
        CatalogPageRow {
            null_bit_map,
            cp_catalog_page_sk,
            cp_catalog_page_id,
            cp_start_date_id,
            cp_end_date_id,
            cp_department,
            cp_catalog_number,
            cp_catalog_page_number,
            cp_description,
            cp_type,
        }
    }

    fn is_null(&self, column: &CatalogPageGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - CatalogPageGeneratorColumn::CpCatalogPageSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(
        &self,
        value: i64,
        column: &CatalogPageGeneratorColumn,
    ) -> String {
        if self.is_null(column) || value < 0 {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null(&self, value: &str, column: &CatalogPageGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_int_or_null(&self, value: i32, column: &CatalogPageGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }
}

impl TableRow for CatalogPageRow {
    fn get_values(&self) -> Vec<String> {
        use CatalogPageGeneratorColumn::*;
        vec![
            self.get_string_or_null_for_key(self.cp_catalog_page_sk, &CpCatalogPageSk),
            self.get_string_or_null(&self.cp_catalog_page_id, &CpCatalogPageId),
            self.get_string_or_null_for_key(self.cp_start_date_id, &CpStartDateId),
            self.get_string_or_null_for_key(self.cp_end_date_id, &CpEndDateId),
            self.get_string_or_null(&self.cp_department, &CpDepartment),
            self.get_int_or_null(self.cp_catalog_number, &CpCatalogNumber),
            self.get_int_or_null(self.cp_catalog_page_number, &CpCatalogPageNumber),
            self.get_string_or_null(&self.cp_description, &CpDescription),
            self.get_string_or_null(&self.cp_type, &CpType),
        ]
    }
}
