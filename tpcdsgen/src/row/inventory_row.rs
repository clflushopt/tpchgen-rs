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
use crate::row::TableRow;

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
