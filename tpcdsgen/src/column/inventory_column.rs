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

//! Inventory column definitions

use crate::column::{Column, ColumnType, ColumnTypes, Table};

/// Columns for the inventory table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryColumn {
    InvDateSk,
    InvItemSk,
    InvWarehouseSk,
    InvQuantityOnHand,
}

impl InventoryColumn {
    /// Get all column variants in order
    pub fn values() -> &'static [InventoryColumn] {
        use InventoryColumn::*;
        static COLUMNS: [InventoryColumn; 4] =
            [InvDateSk, InvItemSk, InvWarehouseSk, InvQuantityOnHand];
        &COLUMNS
    }
}

impl Column for InventoryColumn {
    fn get_table(&self) -> Table {
        Table::Inventory
    }

    fn get_name(&self) -> &'static str {
        match self {
            InventoryColumn::InvDateSk => "inv_date_sk",
            InventoryColumn::InvItemSk => "inv_item_sk",
            InventoryColumn::InvWarehouseSk => "inv_warehouse_sk",
            InventoryColumn::InvQuantityOnHand => "inv_quantity_on_hand",
        }
    }

    fn get_type(&self) -> &ColumnType {
        match self {
            InventoryColumn::InvDateSk => ColumnTypes::identifier(),
            InventoryColumn::InvItemSk => ColumnTypes::identifier(),
            InventoryColumn::InvWarehouseSk => ColumnTypes::identifier(),
            InventoryColumn::InvQuantityOnHand => ColumnTypes::integer(),
        }
    }

    fn get_position(&self) -> i32 {
        match self {
            InventoryColumn::InvDateSk => 0,
            InventoryColumn::InvItemSk => 1,
            InventoryColumn::InvWarehouseSk => 2,
            InventoryColumn::InvQuantityOnHand => 3,
        }
    }
}
