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

//! Store row structure and formatting

use crate::generator::{GeneratorColumn, StoreGeneratorColumn};
use crate::row::TableRow;
use crate::types::{Address, Date, Decimal};

/// Store row
#[derive(Clone)]
pub struct StoreRow {
    null_bit_map: i64,
    store_sk: i64,
    store_id: String,
    rec_start_date_id: i64,
    rec_end_date_id: i64,
    closed_date_id: i64,
    store_name: String,
    employees: i32,
    floor_space: i32,
    hours: String,
    store_manager: String,
    market_id: i32,
    d_tax_percentage: Decimal,
    geography_class: String,
    market_desc: String,
    market_manager: String,
    division_id: i64,
    division_name: String,
    company_id: i64,
    company_name: String,
    address: Address,
}

impl StoreRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        store_sk: i64,
        store_id: String,
        rec_start_date_id: i64,
        rec_end_date_id: i64,
        closed_date_id: i64,
        store_name: String,
        employees: i32,
        floor_space: i32,
        hours: String,
        store_manager: String,
        market_id: i32,
        d_tax_percentage: Decimal,
        geography_class: String,
        market_desc: String,
        market_manager: String,
        division_id: i64,
        division_name: String,
        company_id: i64,
        company_name: String,
        address: Address,
    ) -> Self {
        StoreRow {
            null_bit_map,
            store_sk,
            store_id,
            rec_start_date_id,
            rec_end_date_id,
            closed_date_id,
            store_name,
            employees,
            floor_space,
            hours,
            store_manager,
            market_id,
            d_tax_percentage,
            geography_class,
            market_desc,
            market_manager,
            division_id,
            division_name,
            company_id,
            company_name,
            address,
        }
    }

    fn is_null(&self, column: &StoreGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - StoreGeneratorColumn::WStoreSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(&self, value: i64, column: &StoreGeneratorColumn) -> String {
        if self.is_null(column) || value < 0 {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null(&self, value: &str, column: &StoreGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_int_or_null(&self, value: i32, column: &StoreGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_decimal_or_null(&self, value: &Decimal, column: &StoreGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_date_string_or_null(&self, julian_days: i64, column: &StoreGeneratorColumn) -> String {
        if self.is_null(column) || julian_days < 0 {
            String::new()
        } else {
            Date::from_julian_days(julian_days as i32).to_string()
        }
    }

    // Getters for SCD fields
    pub fn get_closed_date_id(&self) -> i64 {
        self.closed_date_id
    }

    pub fn get_store_name(&self) -> &str {
        &self.store_name
    }

    pub fn get_employees(&self) -> i32 {
        self.employees
    }

    pub fn get_floor_space(&self) -> i32 {
        self.floor_space
    }

    pub fn get_hours(&self) -> &str {
        &self.hours
    }

    pub fn get_store_manager(&self) -> &str {
        &self.store_manager
    }

    pub fn get_market_id(&self) -> i32 {
        self.market_id
    }

    pub fn get_d_tax_percentage(&self) -> Decimal {
        self.d_tax_percentage
    }

    pub fn get_market_desc(&self) -> &str {
        &self.market_desc
    }

    pub fn get_market_manager(&self) -> &str {
        &self.market_manager
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }
}

impl TableRow for StoreRow {
    fn get_values(&self) -> Vec<String> {
        use StoreGeneratorColumn::*;
        vec![
            self.get_string_or_null_for_key(self.store_sk, &WStoreSk),
            self.get_string_or_null(&self.store_id, &WStoreId),
            self.get_date_string_or_null(self.rec_start_date_id, &WStoreRecStartDateId),
            self.get_date_string_or_null(self.rec_end_date_id, &WStoreRecEndDateId),
            self.get_string_or_null_for_key(self.closed_date_id, &WStoreClosedDateId),
            self.get_string_or_null(&self.store_name, &WStoreName),
            self.get_int_or_null(self.employees, &WStoreEmployees),
            self.get_int_or_null(self.floor_space, &WStoreFloorSpace),
            self.get_string_or_null(&self.hours, &WStoreHours),
            self.get_string_or_null(&self.store_manager, &WStoreManager),
            self.get_int_or_null(self.market_id, &WStoreMarketId),
            self.get_string_or_null(&self.geography_class, &WStoreGeographyClass),
            self.get_string_or_null(&self.market_desc, &WStoreMarketDesc),
            self.get_string_or_null(&self.market_manager, &WStoreMarketManager),
            self.get_string_or_null_for_key(self.division_id, &WStoreDivisionId),
            self.get_string_or_null(&self.division_name, &WStoreDivisionName),
            self.get_string_or_null_for_key(self.company_id, &WStoreCompanyId),
            self.get_string_or_null(&self.company_name, &WStoreCompanyName),
            self.get_int_or_null(self.address.get_street_number(), &WStoreAddressStreetNum),
            self.get_string_or_null(&self.address.get_street_name(), &WStoreAddressStreetName1),
            self.get_string_or_null(self.address.get_street_type(), &WStoreAddressStreetType),
            self.get_string_or_null(self.address.get_suite_number(), &WStoreAddressSuiteNum),
            self.get_string_or_null(self.address.get_city(), &WStoreAddressCity),
            self.get_string_or_null(
                self.address.get_county().unwrap_or(""),
                &WStoreAddressCounty,
            ),
            self.get_string_or_null(self.address.get_state(), &WStoreAddressState),
            self.get_string_or_null(&format!("{:05}", self.address.get_zip()), &WStoreAddressZip),
            self.get_string_or_null(self.address.get_country(), &WStoreAddressCountry),
            self.get_string_or_null(
                &self.address.get_gmt_offset().to_string(),
                &WStoreAddressGmtOffset,
            ),
            self.get_decimal_or_null(&self.d_tax_percentage, &WStoreTaxPercentage),
        ]
    }
}
