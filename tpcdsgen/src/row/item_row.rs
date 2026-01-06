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

//! Item row structure and formatting

use crate::generator::{GeneratorColumn, ItemGeneratorColumn};
use crate::row::TableRow;
use crate::types::{Date, Decimal};

/// Item row
#[derive(Clone)]
pub struct ItemRow {
    null_bit_map: i64,
    i_item_sk: i64,
    i_item_id: String,
    i_rec_start_date_id: i64,
    i_rec_end_date_id: i64,
    i_item_desc: String,
    i_current_price: Decimal,
    i_wholesale_cost: Decimal,
    i_brand_id: i64,
    i_brand: String,
    i_class_id: i64,
    i_class: String,
    i_category_id: i64,
    i_category: String,
    i_manufact_id: i64,
    i_manufact: String,
    i_size: String,
    i_formulation: String,
    i_color: String,
    i_units: String,
    i_container: String,
    i_manager_id: i64,
    i_product_name: String,
}

impl ItemRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        i_item_sk: i64,
        i_item_id: String,
        i_rec_start_date_id: i64,
        i_rec_end_date_id: i64,
        i_item_desc: String,
        i_current_price: Decimal,
        i_wholesale_cost: Decimal,
        i_brand_id: i64,
        i_brand: String,
        i_class_id: i64,
        i_class: String,
        i_category_id: i64,
        i_category: String,
        i_manufact_id: i64,
        i_manufact: String,
        i_size: String,
        i_formulation: String,
        i_color: String,
        i_units: String,
        i_container: String,
        i_manager_id: i64,
        i_product_name: String,
    ) -> Self {
        ItemRow {
            null_bit_map,
            i_item_sk,
            i_item_id,
            i_rec_start_date_id,
            i_rec_end_date_id,
            i_item_desc,
            i_current_price,
            i_wholesale_cost,
            i_brand_id,
            i_brand,
            i_class_id,
            i_class,
            i_category_id,
            i_category,
            i_manufact_id,
            i_manufact,
            i_size,
            i_formulation,
            i_color,
            i_units,
            i_container,
            i_manager_id,
            i_product_name,
        }
    }

    fn is_null(&self, column: &ItemGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - ItemGeneratorColumn::IItemSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(&self, value: i64, column: &ItemGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null(&self, value: &str, column: &ItemGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_decimal_or_null(&self, value: &Decimal, column: &ItemGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_date_string_or_null(&self, julian_days: i64, column: &ItemGeneratorColumn) -> String {
        if self.is_null(column) || julian_days < 0 {
            String::new()
        } else {
            Date::from_julian_days(julian_days as i32).to_string()
        }
    }

    // Getters for SCD fields
    pub fn get_i_item_desc(&self) -> &str {
        &self.i_item_desc
    }

    pub fn get_i_wholesale_cost(&self) -> Decimal {
        self.i_wholesale_cost
    }

    pub fn get_i_brand_id(&self) -> i64 {
        self.i_brand_id
    }

    pub fn get_i_class_id(&self) -> i64 {
        self.i_class_id
    }

    pub fn get_i_manufact_id(&self) -> i64 {
        self.i_manufact_id
    }

    pub fn get_i_manufact(&self) -> &str {
        &self.i_manufact
    }

    pub fn get_i_formulation(&self) -> &str {
        &self.i_formulation
    }
}

impl TableRow for ItemRow {
    fn get_values(&self) -> Vec<String> {
        use ItemGeneratorColumn::*;
        vec![
            self.get_string_or_null_for_key(self.i_item_sk, &IItemSk),
            self.get_string_or_null(&self.i_item_id, &IItemId),
            self.get_date_string_or_null(self.i_rec_start_date_id, &IRecStartDateId),
            self.get_date_string_or_null(self.i_rec_end_date_id, &IRecEndDateId),
            self.get_string_or_null(&self.i_item_desc, &IItemDesc),
            self.get_decimal_or_null(&self.i_current_price, &ICurrentPrice),
            self.get_decimal_or_null(&self.i_wholesale_cost, &IWholesaleCost),
            self.get_string_or_null_for_key(self.i_brand_id, &IBrandId),
            self.get_string_or_null(&self.i_brand, &IBrand),
            self.get_string_or_null_for_key(self.i_class_id, &IClassId),
            self.get_string_or_null(&self.i_class, &IClass),
            self.get_string_or_null_for_key(self.i_category_id, &ICategoryId),
            self.get_string_or_null(&self.i_category, &ICategory),
            self.get_string_or_null_for_key(self.i_manufact_id, &IManufactId),
            self.get_string_or_null(&self.i_manufact, &IManufact),
            self.get_string_or_null(&self.i_size, &ISize),
            self.get_string_or_null(&self.i_formulation, &IFormulation),
            self.get_string_or_null(&self.i_color, &IColor),
            self.get_string_or_null(&self.i_units, &IUnits),
            self.get_string_or_null(&self.i_container, &IContainer),
            self.get_string_or_null_for_key(self.i_manager_id, &IManagerId),
            self.get_string_or_null(&self.i_product_name, &IProductName),
        ]
    }
}
