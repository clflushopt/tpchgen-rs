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

//! Customer column definitions (CustomerColumn)

use crate::column::{Column, ColumnType, ColumnTypes, Table};
use std::sync::OnceLock;

/// Customer column enum (CustomerColumn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerColumn {
    CCustomerSk,
    CCustomerId,
    CCurrentCdemoSk,
    CCurrentHdemoSk,
    CCurrentAddrSk,
    CFirstShiptoDateSk,
    CFirstSalesDateSk,
    CSalutation,
    CFirstName,
    CLastName,
    CPreferredCustFlag,
    CBirthDay,
    CBirthMonth,
    CBirthYear,
    CBirthCountry,
    CLogin,
    CEmailAddress,
    CLastReviewDateSk,
}

impl CustomerColumn {
    /// Get all column values in order
    pub fn values() -> &'static [CustomerColumn] {
        use CustomerColumn::*;
        static VALUES: &[CustomerColumn] = &[
            CCustomerSk,
            CCustomerId,
            CCurrentCdemoSk,
            CCurrentHdemoSk,
            CCurrentAddrSk,
            CFirstShiptoDateSk,
            CFirstSalesDateSk,
            CSalutation,
            CFirstName,
            CLastName,
            CPreferredCustFlag,
            CBirthDay,
            CBirthMonth,
            CBirthYear,
            CBirthCountry,
            CLogin,
            CEmailAddress,
            CLastReviewDateSk,
        ];
        VALUES
    }

    /// Get the column type for this column
    fn get_column_type(&self) -> &'static ColumnType {
        use CustomerColumn::*;
        match self {
            CCustomerSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CCustomerId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(16))
            }
            CCurrentCdemoSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CCurrentHdemoSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CCurrentAddrSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CFirstShiptoDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CFirstSalesDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CSalutation => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CFirstName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(20))
            }
            CLastName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(30))
            }
            CPreferredCustFlag => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            CBirthDay => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CBirthMonth => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CBirthYear => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CBirthCountry => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(20))
            }
            CLogin => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(13))
            }
            CEmailAddress => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(50))
            }
            CLastReviewDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
        }
    }
}

impl Column for CustomerColumn {
    fn get_table(&self) -> Table {
        Table::Customer
    }

    fn get_name(&self) -> &'static str {
        use CustomerColumn::*;
        match self {
            CCustomerSk => "c_customer_sk",
            CCustomerId => "c_customer_id",
            CCurrentCdemoSk => "c_current_cdemo_sk",
            CCurrentHdemoSk => "c_current_hdemo_sk",
            CCurrentAddrSk => "c_current_addr_sk",
            CFirstShiptoDateSk => "c_first_shipto_date_sk",
            CFirstSalesDateSk => "c_first_sales_date_sk",
            CSalutation => "c_salutation",
            CFirstName => "c_first_name",
            CLastName => "c_last_name",
            CPreferredCustFlag => "c_preferred_cust_flag",
            CBirthDay => "c_birth_day",
            CBirthMonth => "c_birth_month",
            CBirthYear => "c_birth_year",
            CBirthCountry => "c_birth_country",
            CLogin => "c_login",
            CEmailAddress => "c_email_address",
            CLastReviewDateSk => "c_last_review_date_sk",
        }
    }

    fn get_type(&self) -> &ColumnType {
        self.get_column_type()
    }

    fn get_position(&self) -> i32 {
        use CustomerColumn::*;
        match self {
            CCustomerSk => 0,
            CCustomerId => 1,
            CCurrentCdemoSk => 2,
            CCurrentHdemoSk => 3,
            CCurrentAddrSk => 4,
            CFirstShiptoDateSk => 5,
            CFirstSalesDateSk => 6,
            CSalutation => 7,
            CFirstName => 8,
            CLastName => 9,
            CPreferredCustFlag => 10,
            CBirthDay => 11,
            CBirthMonth => 12,
            CBirthYear => 13,
            CBirthCountry => 14,
            CLogin => 15,
            CEmailAddress => 16,
            CLastReviewDateSk => 17,
        }
    }
}
