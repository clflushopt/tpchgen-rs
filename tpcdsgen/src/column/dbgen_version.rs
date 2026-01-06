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
pub enum DbgenVersionColumn {
    DvVersion,
    DvCreateDate,
    DvCreateTime,
    DvCmdlineArgs,
}

impl DbgenVersionColumn {
    /// Get all column values in order
    pub fn values() -> &'static [DbgenVersionColumn] {
        &[
            DbgenVersionColumn::DvVersion,
            DbgenVersionColumn::DvCreateDate,
            DbgenVersionColumn::DvCreateTime,
            DbgenVersionColumn::DvCmdlineArgs,
        ]
    }
}

impl Column for DbgenVersionColumn {
    fn get_table(&self) -> Table {
        Table::DbgenVersion
    }

    fn get_name(&self) -> &'static str {
        match self {
            DbgenVersionColumn::DvVersion => "dv_version",
            DbgenVersionColumn::DvCreateDate => "dv_create_date",
            DbgenVersionColumn::DvCreateTime => "dv_create_time",
            DbgenVersionColumn::DvCmdlineArgs => "dv_cmdline_args",
        }
    }

    fn get_type(&self) -> &ColumnType {
        use DbgenVersionColumn::*;
        match self {
            DvVersion => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(16))
            }
            DvCreateDate => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::date().clone())
            }
            DvCreateTime => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::time().clone())
            }
            DvCmdlineArgs => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(200))
            }
        }
    }

    fn get_position(&self) -> i32 {
        match self {
            DbgenVersionColumn::DvVersion => 0,
            DbgenVersionColumn::DvCreateDate => 1,
            DbgenVersionColumn::DvCreateTime => 2,
            DbgenVersionColumn::DvCmdlineArgs => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnTypeBase;

    #[test]
    fn test_dbgen_version_column_count() {
        assert_eq!(DbgenVersionColumn::values().len(), 4);
    }

    #[test]
    fn test_dbgen_version_column_names() {
        assert_eq!(DbgenVersionColumn::DvVersion.get_name(), "dv_version");
        assert_eq!(
            DbgenVersionColumn::DvCreateDate.get_name(),
            "dv_create_date"
        );
        assert_eq!(
            DbgenVersionColumn::DvCreateTime.get_name(),
            "dv_create_time"
        );
        assert_eq!(
            DbgenVersionColumn::DvCmdlineArgs.get_name(),
            "dv_cmdline_args"
        );
    }

    #[test]
    fn test_dbgen_version_column_types() {
        assert_eq!(
            DbgenVersionColumn::DvVersion.get_type().get_base(),
            ColumnTypeBase::Varchar
        );
        assert_eq!(
            DbgenVersionColumn::DvCreateDate.get_type().get_base(),
            ColumnTypeBase::Date
        );
        assert_eq!(
            DbgenVersionColumn::DvCreateTime.get_type().get_base(),
            ColumnTypeBase::Time
        );
    }

    #[test]
    fn test_dbgen_version_column_positions() {
        assert_eq!(DbgenVersionColumn::DvVersion.get_position(), 0);
        assert_eq!(DbgenVersionColumn::DvCmdlineArgs.get_position(), 3);
    }

    #[test]
    fn test_dbgen_version_column_table() {
        assert_eq!(
            DbgenVersionColumn::DvVersion.get_table(),
            Table::DbgenVersion
        );
    }
}
