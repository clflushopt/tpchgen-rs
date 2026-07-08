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

//! Inventory row data structure

use crate::generator::{GeneratorColumn, InventoryGeneratorColumn};
use crate::row::table_row::dat_field;
use crate::row::TableRow;
use std::fmt;

/// Represents a single row in the inventory table.
#[derive(Clone)]
pub struct InventoryRow {
    null_bit_map: i64,
    inv_date_sk: i64,
    inv_item_sk: i64,
    inv_warehouse_sk: i64,
    inv_quantity_on_hand: i32,
}

impl InventoryRow {
    pub fn new(
        null_bit_map: i64,
        inv_date_sk: i64,
        inv_item_sk: i64,
        inv_warehouse_sk: i64,
        inv_quantity_on_hand: i32,
    ) -> Self {
        InventoryRow {
            null_bit_map,
            inv_date_sk,
            inv_item_sk,
            inv_warehouse_sk,
            inv_quantity_on_hand,
        }
    }

    fn get_string_or_null_for_key(&self, value: i64, column: InventoryGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null(&self, value: i32, column: InventoryGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn is_null_at(&self, column: InventoryGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - InventoryGeneratorColumn::InvDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_inv_date_sk(&self) -> i64 {
        self.inv_date_sk
    }

    pub fn get_inv_item_sk(&self) -> i64 {
        self.inv_item_sk
    }

    pub fn get_inv_warehouse_sk(&self) -> i64 {
        self.inv_warehouse_sk
    }

    pub fn get_inv_quantity_on_hand(&self) -> i32 {
        self.inv_quantity_on_hand
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for InventoryRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InventoryGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|",
            dat_field(self.inv_date_sk, self.is_null_at(InvDateSk)),
            dat_field(self.inv_item_sk, self.is_null_at(InvItemSk)),
            dat_field(self.inv_warehouse_sk, self.is_null_at(InvWarehouseSk)),
            dat_field(
                self.inv_quantity_on_hand,
                self.is_null_at(InvQuantityOnHand)
            ),
        )
    }
}

impl TableRow for InventoryRow {
    fn get_values(&self) -> Vec<String> {
        vec![
            self.get_string_or_null_for_key(self.inv_date_sk, InventoryGeneratorColumn::InvDateSk),
            self.get_string_or_null_for_key(self.inv_item_sk, InventoryGeneratorColumn::InvItemSk),
            self.get_string_or_null_for_key(
                self.inv_warehouse_sk,
                InventoryGeneratorColumn::InvWarehouseSk,
            ),
            self.get_string_or_null(
                self.inv_quantity_on_hand,
                InventoryGeneratorColumn::InvQuantityOnHand,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        let row = InventoryRow::new(0b10, 2451545, 17, 3, 250);
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
