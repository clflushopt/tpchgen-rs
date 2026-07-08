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

//! Customer row definition (CustomerRow)

use crate::generator::CustomerGeneratorColumn;
use crate::row::table_row::{dat_bool, dat_field, DatField};
use crate::row::TableRow;
use std::fmt;

/// Customer row (CustomerRow)
#[derive(Debug, Clone)]
pub struct CustomerRow {
    null_bit_map: i64,
    c_customer_sk: i64,
    c_customer_id: String,
    c_current_cdemo_sk: i64,
    c_current_hdemo_sk: i64,
    c_current_addr_sk: i64,
    c_first_shipto_date_id: i32,
    c_first_sales_date_id: i32,
    c_salutation: String,
    c_first_name: String,
    c_last_name: String,
    c_preferred_cust_flag: bool,
    c_birth_day: i32,
    c_birth_month: i32,
    c_birth_year: i32,
    c_birth_country: String,
    c_login: Option<String>, // always null in the Java implementation
    c_email_address: String,
    c_last_review_date: i32,
}

impl CustomerRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c_customer_sk: i64,
        c_customer_id: String,
        c_current_cdemo_sk: i64,
        c_current_hdemo_sk: i64,
        c_current_addr_sk: i64,
        c_first_shipto_date_id: i32,
        c_first_sales_date_id: i32,
        c_salutation: String,
        c_first_name: String,
        c_last_name: String,
        c_preferred_cust_flag: bool,
        c_birth_day: i32,
        c_birth_month: i32,
        c_birth_year: i32,
        c_birth_country: String,
        c_email_address: String,
        c_last_review_date: i32,
        null_bit_map: i64,
    ) -> Self {
        CustomerRow {
            null_bit_map,
            c_customer_sk,
            c_customer_id,
            c_current_cdemo_sk,
            c_current_hdemo_sk,
            c_current_addr_sk,
            c_first_shipto_date_id,
            c_first_sales_date_id,
            c_salutation,
            c_first_name,
            c_last_name,
            c_preferred_cust_flag,
            c_birth_day,
            c_birth_month,
            c_birth_year,
            c_birth_country,
            c_login: None, // never gets set to anything
            c_email_address,
            c_last_review_date,
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_c_customer_sk(&self) -> i64 {
        self.c_customer_sk
    }

    pub fn get_c_customer_id(&self) -> &str {
        &self.c_customer_id
    }

    pub fn get_c_current_cdemo_sk(&self) -> i64 {
        self.c_current_cdemo_sk
    }

    pub fn get_c_current_hdemo_sk(&self) -> i64 {
        self.c_current_hdemo_sk
    }

    pub fn get_c_current_addr_sk(&self) -> i64 {
        self.c_current_addr_sk
    }

    pub fn get_c_first_shipto_date_id(&self) -> i32 {
        self.c_first_shipto_date_id
    }

    pub fn get_c_first_sales_date_id(&self) -> i32 {
        self.c_first_sales_date_id
    }

    pub fn get_c_salutation(&self) -> &str {
        &self.c_salutation
    }

    pub fn get_c_first_name(&self) -> &str {
        &self.c_first_name
    }

    pub fn get_c_last_name(&self) -> &str {
        &self.c_last_name
    }

    pub fn get_c_preferred_cust_flag(&self) -> bool {
        self.c_preferred_cust_flag
    }

    pub fn get_c_birth_day(&self) -> i32 {
        self.c_birth_day
    }

    pub fn get_c_birth_month(&self) -> i32 {
        self.c_birth_month
    }

    pub fn get_c_birth_year(&self) -> i32 {
        self.c_birth_year
    }

    pub fn get_c_birth_country(&self) -> &str {
        &self.c_birth_country
    }

    pub fn get_c_login(&self) -> Option<&str> {
        self.c_login.as_deref()
    }

    pub fn get_c_email_address(&self) -> &str {
        &self.c_email_address
    }

    pub fn get_c_last_review_date(&self) -> i32 {
        self.c_last_review_date
    }

    /// Check if a column is null based on the null bit map
    fn is_null(&self, column: CustomerGeneratorColumn) -> bool {
        let position = column.get_global_column_number()
            - CustomerGeneratorColumn::CCustomerSk.get_global_column_number();
        (self.null_bit_map & (1 << position)) != 0
    }

    /// Get string or null for key fields (surrogate keys)
    fn get_string_or_null_for_key(&self, value: i64, column: CustomerGeneratorColumn) -> String {
        if self.is_null(column) || value < 0 {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Get string or null for string fields
    fn get_string_or_null(&self, value: &str, column: CustomerGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Get string or null for integer fields
    fn get_string_or_null_for_int(&self, value: i32, column: CustomerGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Get string or null for boolean fields (Y/N format)
    fn get_string_or_null_for_boolean(
        &self,
        value: bool,
        column: CustomerGeneratorColumn,
    ) -> String {
        if self.is_null(column) {
            String::new()
        } else if value {
            "Y".to_string()
        } else {
            "N".to_string()
        }
    }
}

impl CustomerRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative (mirrors `get_string_or_null_for_key`).
    fn key_field(&self, value: i64, column: CustomerGeneratorColumn) -> DatField<i64> {
        dat_field(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set
    /// (mirrors the remaining `get_string_or_null*` helpers).
    fn field<T>(&self, value: T, column: CustomerGeneratorColumn) -> DatField<T> {
        dat_field(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for CustomerRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CustomerGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.c_customer_sk, CCustomerSk),
            self.field(&self.c_customer_id, CCustomerId),
            self.key_field(self.c_current_cdemo_sk, CCurrentCdemoSk),
            self.key_field(self.c_current_hdemo_sk, CCurrentHdemoSk),
            self.key_field(self.c_current_addr_sk, CCurrentAddrSk),
            self.field(self.c_first_shipto_date_id, CFirstShiptoDateId),
            self.field(self.c_first_sales_date_id, CFirstSalesDateId),
            self.field(&self.c_salutation, CSalutation),
            self.field(&self.c_first_name, CFirstName),
            self.field(&self.c_last_name, CLastName),
            dat_bool(self.c_preferred_cust_flag, self.is_null(CPreferredCustFlag)),
            self.field(self.c_birth_day, CBirthDay),
            self.field(self.c_birth_month, CBirthMonth),
            self.field(self.c_birth_year, CBirthYear),
            self.field(&self.c_birth_country, CBirthCountry),
            // c_login is emitted without a null check, like get_values()
            self.c_login.as_deref().unwrap_or_default(),
            self.field(&self.c_email_address, CEmailAddress),
            self.field(self.c_last_review_date, CLastReviewDate),
        )
    }
}

impl TableRow for CustomerRow {
    fn get_values(&self) -> Vec<String> {
        use CustomerGeneratorColumn::*;

        vec![
            self.get_string_or_null_for_key(self.c_customer_sk, CCustomerSk),
            self.get_string_or_null(&self.c_customer_id, CCustomerId),
            self.get_string_or_null_for_key(self.c_current_cdemo_sk, CCurrentCdemoSk),
            self.get_string_or_null_for_key(self.c_current_hdemo_sk, CCurrentHdemoSk),
            self.get_string_or_null_for_key(self.c_current_addr_sk, CCurrentAddrSk),
            self.get_string_or_null_for_int(self.c_first_shipto_date_id, CFirstShiptoDateId),
            self.get_string_or_null_for_int(self.c_first_sales_date_id, CFirstSalesDateId),
            self.get_string_or_null(&self.c_salutation, CSalutation),
            self.get_string_or_null(&self.c_first_name, CFirstName),
            self.get_string_or_null(&self.c_last_name, CLastName),
            self.get_string_or_null_for_boolean(self.c_preferred_cust_flag, CPreferredCustFlag),
            self.get_string_or_null_for_int(self.c_birth_day, CBirthDay),
            self.get_string_or_null_for_int(self.c_birth_month, CBirthMonth),
            self.get_string_or_null_for_int(self.c_birth_year, CBirthYear),
            self.get_string_or_null(&self.c_birth_country, CBirthCountry),
            self.c_login.clone().unwrap_or_default(), // always null/empty
            self.get_string_or_null(&self.c_email_address, CEmailAddress),
            self.get_string_or_null_for_int(self.c_last_review_date, CLastReviewDate),
        ]
    }
}

use crate::generator::GeneratorColumn;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        // Null bit on c_customer_id (position 1) plus a negative key
        // (c_current_hdemo_sk) exercise both NULL paths.
        let row = CustomerRow::new(
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            100,
            -1,
            400,
            2450815,
            2450820,
            "Sir".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            true,
            14,
            7,
            1970,
            "CHILE".to_string(),
            "John.Doe@example.com".to_string(),
            2452293,
            0b10,
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
