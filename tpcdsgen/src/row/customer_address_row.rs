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

//! Customer address row (CustomerAddressRow)

use crate::row::TableRow;
use crate::types::Address;

/// Customer address table row (CustomerAddressRow)
#[derive(Debug, Clone)]
pub struct CustomerAddressRow {
    null_bit_map: i64,
    ca_addr_sk: i64,
    ca_addr_id: String,
    ca_address: Address,
    ca_location_type: String,
}

impl CustomerAddressRow {
    pub fn new(
        null_bit_map: i64,
        ca_addr_sk: i64,
        ca_addr_id: String,
        ca_address: Address,
        ca_location_type: String,
    ) -> Self {
        CustomerAddressRow {
            null_bit_map,
            ca_addr_sk,
            ca_addr_id,
            ca_address,
            ca_location_type,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    /// Convert value to string or empty string if null (getStringOrNull)
    fn get_string_or_null<T: ToString>(&self, value: T, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Convert key to string or empty string if null (getStringOrNullForKey)
    fn get_string_or_null_for_key(&self, value: i64, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            value.to_string()
        }
    }

    pub fn get_ca_addr_sk(&self) -> i64 {
        self.ca_addr_sk
    }

    pub fn get_ca_addr_id(&self) -> &str {
        &self.ca_addr_id
    }

    pub fn get_ca_address(&self) -> &Address {
        &self.ca_address
    }

    pub fn get_ca_location_type(&self) -> &str {
        &self.ca_location_type
    }
}

impl TableRow for CustomerAddressRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java CustomerAddressColumn ordinals (0-12)
        vec![
            self.get_string_or_null_for_key(self.ca_addr_sk, 0), // CA_ADDRESS_SK
            self.get_string_or_null(&self.ca_addr_id, 1),        // CA_ADDRESS_ID
            self.get_string_or_null(self.ca_address.get_street_number(), 2), // CA_STREET_NUMBER
            self.get_string_or_null(self.ca_address.get_street_name(), 3), // CA_STREET_NAME
            self.get_string_or_null(self.ca_address.get_street_type(), 4), // CA_STREET_TYPE
            self.get_string_or_null(self.ca_address.get_suite_number(), 5), // CA_SUITE_NUMBER
            self.get_string_or_null(self.ca_address.get_city(), 6), // CA_CITY
            self.get_string_or_null(self.ca_address.get_county().unwrap_or(""), 7), // CA_COUNTY
            self.get_string_or_null(self.ca_address.get_state(), 8), // CA_STATE
            self.get_string_or_null(format!("{:05}", self.ca_address.get_zip()), 9), // CA_ZIP
            self.get_string_or_null(self.ca_address.get_country(), 10), // CA_COUNTRY
            self.get_string_or_null(self.ca_address.get_gmt_offset(), 11), // CA_GMT_OFFSET
            self.get_string_or_null(&self.ca_location_type, 12), // CA_LOCATION_TYPE
        ]
    }
}
