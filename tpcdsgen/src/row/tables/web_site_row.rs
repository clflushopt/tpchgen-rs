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

use crate::generator::{GeneratorColumn, WebSiteGeneratorColumn};
use crate::row::table_row::{dat_field, dat_opt, dat_zip, DatField};
use crate::row::TableRow;
use crate::types::{Address, Date, Decimal};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct WebSiteRow {
    null_bit_map: i64,
    web_site_sk: i64,
    web_site_id: String,
    web_rec_start_date_id: i64,
    web_rec_end_date_id: i64,
    web_name: String,
    web_open_date: i64,
    web_close_date: i64,
    web_class: String,
    web_manager: String,
    web_market_id: i32,
    web_market_class: String,
    web_market_desc: String,
    web_market_manager: String,
    web_company_id: i32,
    web_company_name: String,
    web_address: Address,
    web_tax_percentage: Decimal,
}

impl WebSiteRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        web_site_sk: i64,
        web_site_id: String,
        web_rec_start_date_id: i64,
        web_rec_end_date_id: i64,
        web_name: String,
        web_open_date: i64,
        web_close_date: i64,
        web_class: String,
        web_manager: String,
        web_market_id: i32,
        web_market_class: String,
        web_market_desc: String,
        web_market_manager: String,
        web_company_id: i32,
        web_company_name: String,
        web_address: Address,
        web_tax_percentage: Decimal,
    ) -> Self {
        WebSiteRow {
            null_bit_map,
            web_site_sk,
            web_site_id,
            web_rec_start_date_id,
            web_rec_end_date_id,
            web_name,
            web_open_date,
            web_close_date,
            web_class,
            web_manager,
            web_market_id,
            web_market_class,
            web_market_desc,
            web_market_manager,
            web_company_id,
            web_company_name,
            web_address,
            web_tax_percentage,
        }
    }

    // Getters for SCD logic
    pub fn web_name(&self) -> &str {
        &self.web_name
    }

    pub fn web_open_date(&self) -> i64 {
        self.web_open_date
    }

    pub fn web_close_date(&self) -> i64 {
        self.web_close_date
    }

    pub fn web_class(&self) -> &str {
        &self.web_class
    }

    pub fn web_manager(&self) -> &str {
        &self.web_manager
    }

    pub fn web_market_id(&self) -> i32 {
        self.web_market_id
    }

    pub fn web_market_class(&self) -> &str {
        &self.web_market_class
    }

    pub fn web_market_desc(&self) -> &str {
        &self.web_market_desc
    }

    pub fn web_market_manager(&self) -> &str {
        &self.web_market_manager
    }

    pub fn web_company_id(&self) -> i32 {
        self.web_company_id
    }

    pub fn web_company_name(&self) -> &str {
        &self.web_company_name
    }

    pub fn web_address(&self) -> &Address {
        &self.web_address
    }

    pub fn web_tax_percentage(&self) -> &Decimal {
        &self.web_tax_percentage
    }

    fn get_string_or_null_for_key(&self, key: i64, column: WebSiteGeneratorColumn) -> String {
        if key == -1 || self.is_null_at(column) {
            String::new()
        } else {
            key.to_string()
        }
    }

    fn get_string_or_null_string(&self, value: &str, column: WebSiteGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_int(&self, value: i32, column: WebSiteGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_decimal(
        &self,
        value: &Decimal,
        column: WebSiteGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_date_string_or_null_from_julian_days(
        &self,
        julian_days: i64,
        column: WebSiteGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) || julian_days < 0 {
            String::new()
        } else {
            Date::from_julian_days(julian_days as i32).to_string()
        }
    }

    fn is_null_at(&self, column: WebSiteGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - WebSiteGeneratorColumn::WebSiteSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_web_site_sk(&self) -> i64 {
        self.web_site_sk
    }

    pub fn get_web_site_id(&self) -> &str {
        &self.web_site_id
    }

    pub fn get_web_rec_start_date_id(&self) -> i64 {
        self.web_rec_start_date_id
    }

    pub fn get_web_rec_end_date_id(&self) -> i64 {
        self.web_rec_end_date_id
    }
}

impl WebSiteRow {
    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is -1 (mirrors `get_string_or_null_for_key`).
    fn key_field(&self, key: i64, column: WebSiteGeneratorColumn) -> DatField<i64> {
        dat_field(key, key == -1 || self.is_null_at(column))
    }

    /// DAT field for a regular value: NULL when the null bit is set.
    fn field<T>(&self, value: T, column: WebSiteGeneratorColumn) -> DatField<T> {
        dat_field(value, self.is_null_at(column))
    }

    /// DAT field for an SCD date: NULL when the null bit is set or the
    /// julian day is negative (mirrors `get_date_string_or_null_from_julian_days`).
    fn date_field(&self, julian_days: i64, column: WebSiteGeneratorColumn) -> DatField<Date> {
        dat_opt(
            (!(self.is_null_at(column) || julian_days < 0))
                .then(|| Date::from_julian_days(julian_days as i32)),
        )
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for WebSiteRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WebSiteGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.web_site_sk, WebSiteSk),
            self.field(&self.web_site_id, WebSiteId),
            self.date_field(self.web_rec_start_date_id, WebRecStartDateId),
            self.date_field(self.web_rec_end_date_id, WebRecEndDateId),
            self.field(&self.web_name, WebName),
            self.key_field(self.web_open_date, WebOpenDate),
            self.key_field(self.web_close_date, WebCloseDate),
            self.field(&self.web_class, WebClass),
            self.field(&self.web_manager, WebManager),
            self.field(self.web_market_id, WebMarketId),
            self.field(&self.web_market_class, WebMarketClass),
            self.field(&self.web_market_desc, WebMarketDesc),
            self.field(&self.web_market_manager, WebMarketManager),
            self.field(self.web_company_id, WebCompanyId),
            self.field(&self.web_company_name, WebCompanyName),
            self.field(self.web_address.get_street_number(), WebAddressStreetNum),
            self.field(self.web_address.get_street_name(), WebAddressStreetName1),
            self.field(self.web_address.get_street_type(), WebAddressStreetType),
            self.field(self.web_address.get_suite_number(), WebAddressSuiteNum),
            self.field(self.web_address.get_city(), WebAddressCity),
            self.field(
                self.web_address.get_county().unwrap_or(""),
                WebAddressCounty
            ),
            self.field(self.web_address.get_state(), WebAddressState),
            dat_zip(self.web_address.get_zip(), self.is_null_at(WebAddressZip)),
            self.field(self.web_address.get_country(), WebAddressCountry),
            self.field(self.web_address.get_gmt_offset(), WebAddressGmtOffset),
            self.field(self.web_tax_percentage, WebTaxPercentage),
        )
    }
}

impl TableRow for WebSiteRow {
    fn get_values(&self) -> Vec<String> {
        vec![
            self.get_string_or_null_for_key(self.web_site_sk, WebSiteGeneratorColumn::WebSiteSk),
            self.get_string_or_null_string(&self.web_site_id, WebSiteGeneratorColumn::WebSiteId),
            self.get_date_string_or_null_from_julian_days(
                self.web_rec_start_date_id,
                WebSiteGeneratorColumn::WebRecStartDateId,
            ),
            self.get_date_string_or_null_from_julian_days(
                self.web_rec_end_date_id,
                WebSiteGeneratorColumn::WebRecEndDateId,
            ),
            self.get_string_or_null_string(&self.web_name, WebSiteGeneratorColumn::WebName),
            self.get_string_or_null_for_key(
                self.web_open_date,
                WebSiteGeneratorColumn::WebOpenDate,
            ),
            self.get_string_or_null_for_key(
                self.web_close_date,
                WebSiteGeneratorColumn::WebCloseDate,
            ),
            self.get_string_or_null_string(&self.web_class, WebSiteGeneratorColumn::WebClass),
            self.get_string_or_null_string(&self.web_manager, WebSiteGeneratorColumn::WebManager),
            self.get_string_or_null_int(self.web_market_id, WebSiteGeneratorColumn::WebMarketId),
            self.get_string_or_null_string(
                &self.web_market_class,
                WebSiteGeneratorColumn::WebMarketClass,
            ),
            self.get_string_or_null_string(
                &self.web_market_desc,
                WebSiteGeneratorColumn::WebMarketDesc,
            ),
            self.get_string_or_null_string(
                &self.web_market_manager,
                WebSiteGeneratorColumn::WebMarketManager,
            ),
            self.get_string_or_null_int(self.web_company_id, WebSiteGeneratorColumn::WebCompanyId),
            self.get_string_or_null_string(
                &self.web_company_name,
                WebSiteGeneratorColumn::WebCompanyName,
            ),
            self.get_string_or_null_string(
                &self.web_address.get_street_number().to_string(),
                WebSiteGeneratorColumn::WebAddressStreetNum,
            ),
            self.get_string_or_null_string(
                &self.web_address.get_street_name(),
                WebSiteGeneratorColumn::WebAddressStreetName1,
            ),
            self.get_string_or_null_string(
                self.web_address.get_street_type(),
                WebSiteGeneratorColumn::WebAddressStreetType,
            ),
            self.get_string_or_null_string(
                self.web_address.get_suite_number(),
                WebSiteGeneratorColumn::WebAddressSuiteNum,
            ),
            self.get_string_or_null_string(
                self.web_address.get_city(),
                WebSiteGeneratorColumn::WebAddressCity,
            ),
            self.get_string_or_null_string(
                self.web_address.get_county().unwrap_or(""),
                WebSiteGeneratorColumn::WebAddressCounty,
            ),
            self.get_string_or_null_string(
                self.web_address.get_state(),
                WebSiteGeneratorColumn::WebAddressState,
            ),
            self.get_string_or_null_string(
                &format!("{:05}", self.web_address.get_zip()),
                WebSiteGeneratorColumn::WebAddressZip,
            ),
            self.get_string_or_null_string(
                self.web_address.get_country(),
                WebSiteGeneratorColumn::WebAddressCountry,
            ),
            self.get_string_or_null_int(
                self.web_address.get_gmt_offset(),
                WebSiteGeneratorColumn::WebAddressGmtOffset,
            ),
            self.get_string_or_null_decimal(
                &self.web_tax_percentage,
                WebSiteGeneratorColumn::WebTaxPercentage,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_address() -> Address {
        Address::new(
            "Suite 100".to_string(),
            100,
            "Main".to_string(),
            "Street".to_string(),
            "Blvd".to_string(),
            "Springfield".to_string(),
            Some("Some County".to_string()),
            "CA".to_string(),
            "United States".to_string(),
            904,
            -8,
        )
        .unwrap()
    }

    #[test]
    fn test_display_matches_get_values() {
        let row = WebSiteRow::new(
            0b10,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            2450815,
            -1,
            "site_0".to_string(),
            2450807,
            -1,
            "Unknown".to_string(),
            "Otis Boyd".to_string(),
            2,
            "mkt class".to_string(),
            "mkt desc".to_string(),
            "Mkt Mgr".to_string(),
            10,
            "Company".to_string(),
            test_address(),
            Decimal::new(12, 2).unwrap(),
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }

    #[test]
    fn test_web_site_row_values_count() {
        let address = Address::new(
            "Suite 1".to_string(),
            100,
            "Main St".to_string(),
            String::new(),
            "Avenue".to_string(),
            "Springfield".to_string(),
            Some("Sangamon".to_string()),
            "IL".to_string(),
            "United States".to_string(),
            62701,
            -600,
        )
        .unwrap();

        let row = WebSiteRow::new(
            0,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            2450815,
            2451179,
            "site_0".to_string(),
            2450820,
            -1,
            "Unknown".to_string(),
            "John Doe".to_string(),
            1,
            "Market class".to_string(),
            "Market description".to_string(),
            "Jane Smith".to_string(),
            1,
            "Company A".to_string(),
            address,
            Decimal::new(650, 2).unwrap(),
        );

        let values = row.get_values();
        assert_eq!(values.len(), 26);
    }
}
