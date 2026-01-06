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

/// Household Demographics table columns (HouseholdDemographicsColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HouseholdDemographicsColumn {
    HdDemoSk,
    HdIncomeBandSk,
    HdBuyPotential,
    HdDepCount,
    HdVehicleCount,
}

impl HouseholdDemographicsColumn {
    /// Get all columns in order
    pub fn values() -> &'static [HouseholdDemographicsColumn] {
        use HouseholdDemographicsColumn::*;
        static VALUES: &[HouseholdDemographicsColumn] = &[
            HdDemoSk,
            HdIncomeBandSk,
            HdBuyPotential,
            HdDepCount,
            HdVehicleCount,
        ];
        VALUES
    }

    /// Get the column type for this column
    fn get_column_type(&self) -> &'static ColumnType {
        match self {
            HouseholdDemographicsColumn::HdDemoSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            HouseholdDemographicsColumn::HdIncomeBandSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            HouseholdDemographicsColumn::HdBuyPotential => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(15))
            }
            HouseholdDemographicsColumn::HdDepCount => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            HouseholdDemographicsColumn::HdVehicleCount => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
        }
    }
}

impl Column for HouseholdDemographicsColumn {
    fn get_table(&self) -> Table {
        Table::HouseholdDemographics
    }

    fn get_name(&self) -> &'static str {
        match self {
            HouseholdDemographicsColumn::HdDemoSk => "hd_demo_sk",
            HouseholdDemographicsColumn::HdIncomeBandSk => "hd_income_band_sk",
            HouseholdDemographicsColumn::HdBuyPotential => "hd_buy_potential",
            HouseholdDemographicsColumn::HdDepCount => "hd_dep_count",
            HouseholdDemographicsColumn::HdVehicleCount => "hd_vehicle_count",
        }
    }

    fn get_type(&self) -> &ColumnType {
        self.get_column_type()
    }

    fn get_position(&self) -> i32 {
        *self as i32
    }
}
