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

//! Customer address column definitions (CustomerAddressColumn)

use crate::column::{Column, ColumnType, ColumnTypes, Table};
use std::sync::OnceLock;

/// Customer address column enum (CustomerAddressColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerAddressColumn {
    CaAddressSk,
    CaAddressId,
    CaStreetNumber,
    CaStreetName,
    CaStreetType,
    CaSuiteNumber,
    CaCity,
    CaCounty,
    CaState,
    CaZip,
    CaCountry,
    CaGmtOffset,
    CaLocationType,
}

impl CustomerAddressColumn {
    /// Get all column values in order
    pub fn values() -> &'static [CustomerAddressColumn] {
        use CustomerAddressColumn::*;
        static VALUES: &[CustomerAddressColumn] = &[
            CaAddressSk,
            CaAddressId,
            CaStreetNumber,
            CaStreetName,
            CaStreetType,
            CaSuiteNumber,
            CaCity,
            CaCounty,
            CaState,
            CaZip,
            CaCountry,
            CaGmtOffset,
            CaLocationType,
        ];
        VALUES
    }

    /// Get the column type for this column
    fn get_column_type(&self) -> &'static ColumnType {
        use CustomerAddressColumn::*;
        match self {
            CaAddressSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CaAddressId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(16))
            }
            CaStreetNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CaStreetName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            CaStreetType => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(15))
            }
            CaSuiteNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CaCity => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            CaCounty => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(30))
            }
            CaState => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(2))
            }
            CaZip => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CaCountry => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(20))
            }
            CaGmtOffset => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(5, 2))
            }
            CaLocationType => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(20))
            }
        }
    }
}

impl Column for CustomerAddressColumn {
    fn get_table(&self) -> Table {
        Table::CustomerAddress
    }

    fn get_name(&self) -> &'static str {
        use CustomerAddressColumn::*;
        match self {
            CaAddressSk => "ca_address_sk",
            CaAddressId => "ca_address_id",
            CaStreetNumber => "ca_street_number",
            CaStreetName => "ca_street_name",
            CaStreetType => "ca_street_type",
            CaSuiteNumber => "ca_suite_number",
            CaCity => "ca_city",
            CaCounty => "ca_county",
            CaState => "ca_state",
            CaZip => "ca_zip",
            CaCountry => "ca_country",
            CaGmtOffset => "ca_gmt_offset",
            CaLocationType => "ca_location_type",
        }
    }

    fn get_type(&self) -> &ColumnType {
        self.get_column_type()
    }

    fn get_position(&self) -> i32 {
        use CustomerAddressColumn::*;
        match self {
            CaAddressSk => 0,
            CaAddressId => 1,
            CaStreetNumber => 2,
            CaStreetName => 3,
            CaStreetType => 4,
            CaSuiteNumber => 5,
            CaCity => 6,
            CaCounty => 7,
            CaState => 8,
            CaZip => 9,
            CaCountry => 10,
            CaGmtOffset => 11,
            CaLocationType => 12,
        }
    }
}
