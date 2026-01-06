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

use crate::column::{Column, ColumnType, ColumnTypes, Table};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebSiteColumn {
    WebSiteSk,
    WebSiteId,
    WebRecStartDate,
    WebRecEndDate,
    WebName,
    WebOpenDateSk,
    WebCloseDateSk,
    WebClass,
    WebManager,
    WebMktId,
    WebMktClass,
    WebMktDesc,
    WebMarketManager,
    WebCompanyId,
    WebCompanyName,
    WebStreetNumber,
    WebStreetName,
    WebStreetType,
    WebSuiteNumber,
    WebCity,
    WebCounty,
    WebState,
    WebZip,
    WebCountry,
    WebGmtOffset,
    WebTaxPercentage,
}

impl WebSiteColumn {
    pub fn values() -> &'static [WebSiteColumn] {
        &[
            WebSiteColumn::WebSiteSk,
            WebSiteColumn::WebSiteId,
            WebSiteColumn::WebRecStartDate,
            WebSiteColumn::WebRecEndDate,
            WebSiteColumn::WebName,
            WebSiteColumn::WebOpenDateSk,
            WebSiteColumn::WebCloseDateSk,
            WebSiteColumn::WebClass,
            WebSiteColumn::WebManager,
            WebSiteColumn::WebMktId,
            WebSiteColumn::WebMktClass,
            WebSiteColumn::WebMktDesc,
            WebSiteColumn::WebMarketManager,
            WebSiteColumn::WebCompanyId,
            WebSiteColumn::WebCompanyName,
            WebSiteColumn::WebStreetNumber,
            WebSiteColumn::WebStreetName,
            WebSiteColumn::WebStreetType,
            WebSiteColumn::WebSuiteNumber,
            WebSiteColumn::WebCity,
            WebSiteColumn::WebCounty,
            WebSiteColumn::WebState,
            WebSiteColumn::WebZip,
            WebSiteColumn::WebCountry,
            WebSiteColumn::WebGmtOffset,
            WebSiteColumn::WebTaxPercentage,
        ]
    }
}

impl Column for WebSiteColumn {
    fn get_table(&self) -> Table {
        Table::WebSite
    }

    fn get_name(&self) -> &'static str {
        match self {
            WebSiteColumn::WebSiteSk => "web_site_sk",
            WebSiteColumn::WebSiteId => "web_site_id",
            WebSiteColumn::WebRecStartDate => "web_rec_start_date",
            WebSiteColumn::WebRecEndDate => "web_rec_end_date",
            WebSiteColumn::WebName => "web_name",
            WebSiteColumn::WebOpenDateSk => "web_open_date_sk",
            WebSiteColumn::WebCloseDateSk => "web_close_date_sk",
            WebSiteColumn::WebClass => "web_class",
            WebSiteColumn::WebManager => "web_manager",
            WebSiteColumn::WebMktId => "web_mkt_id",
            WebSiteColumn::WebMktClass => "web_mkt_class",
            WebSiteColumn::WebMktDesc => "web_mkt_desc",
            WebSiteColumn::WebMarketManager => "web_market_manager",
            WebSiteColumn::WebCompanyId => "web_company_id",
            WebSiteColumn::WebCompanyName => "web_company_name",
            WebSiteColumn::WebStreetNumber => "web_street_number",
            WebSiteColumn::WebStreetName => "web_street_name",
            WebSiteColumn::WebStreetType => "web_street_type",
            WebSiteColumn::WebSuiteNumber => "web_suite_number",
            WebSiteColumn::WebCity => "web_city",
            WebSiteColumn::WebCounty => "web_county",
            WebSiteColumn::WebState => "web_state",
            WebSiteColumn::WebZip => "web_zip",
            WebSiteColumn::WebCountry => "web_country",
            WebSiteColumn::WebGmtOffset => "web_gmt_offset",
            WebSiteColumn::WebTaxPercentage => "web_tax_percentage",
        }
    }

    fn get_type(&self) -> &ColumnType {
        use WebSiteColumn::*;
        match self {
            WebSiteSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            WebSiteId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(16))
            }
            WebRecStartDate => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::date().clone())
            }
            WebRecEndDate => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::date().clone())
            }
            WebName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            WebOpenDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            WebCloseDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            WebClass => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            WebManager => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(40))
            }
            WebMktId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            WebMktClass => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            WebMktDesc => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(100))
            }
            WebMarketManager => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(40))
            }
            WebCompanyId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            WebCompanyName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(50))
            }
            WebStreetNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            WebStreetName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            WebStreetType => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(15))
            }
            WebSuiteNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            WebCity => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            WebCounty => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(30))
            }
            WebState => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(2))
            }
            WebZip => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            WebCountry => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(20))
            }
            WebGmtOffset => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(5, 2))
            }
            WebTaxPercentage => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(5, 2))
            }
        }
    }

    fn get_position(&self) -> i32 {
        match self {
            WebSiteColumn::WebSiteSk => 0,
            WebSiteColumn::WebSiteId => 1,
            WebSiteColumn::WebRecStartDate => 2,
            WebSiteColumn::WebRecEndDate => 3,
            WebSiteColumn::WebName => 4,
            WebSiteColumn::WebOpenDateSk => 5,
            WebSiteColumn::WebCloseDateSk => 6,
            WebSiteColumn::WebClass => 7,
            WebSiteColumn::WebManager => 8,
            WebSiteColumn::WebMktId => 9,
            WebSiteColumn::WebMktClass => 10,
            WebSiteColumn::WebMktDesc => 11,
            WebSiteColumn::WebMarketManager => 12,
            WebSiteColumn::WebCompanyId => 13,
            WebSiteColumn::WebCompanyName => 14,
            WebSiteColumn::WebStreetNumber => 15,
            WebSiteColumn::WebStreetName => 16,
            WebSiteColumn::WebStreetType => 17,
            WebSiteColumn::WebSuiteNumber => 18,
            WebSiteColumn::WebCity => 19,
            WebSiteColumn::WebCounty => 20,
            WebSiteColumn::WebState => 21,
            WebSiteColumn::WebZip => 22,
            WebSiteColumn::WebCountry => 23,
            WebSiteColumn::WebGmtOffset => 24,
            WebSiteColumn::WebTaxPercentage => 25,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_site_column_count() {
        assert_eq!(WebSiteColumn::values().len(), 26);
    }

    #[test]
    fn test_web_site_column_names() {
        assert_eq!(WebSiteColumn::WebSiteSk.get_name(), "web_site_sk");
        assert_eq!(WebSiteColumn::WebSiteId.get_name(), "web_site_id");
        assert_eq!(
            WebSiteColumn::WebRecStartDate.get_name(),
            "web_rec_start_date"
        );
    }

    #[test]
    fn test_web_site_column_positions() {
        assert_eq!(WebSiteColumn::WebSiteSk.get_position(), 0);
        assert_eq!(WebSiteColumn::WebTaxPercentage.get_position(), 25);
    }
}
