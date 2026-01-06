use crate::column::{Column, ColumnType, ColumnTypes, Table};
use std::sync::OnceLock;

/// Call Center table columns (CallCenterColumn enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallCenterColumn {
    CcCallCenterSk,
    CcCallCenterId,
    CcRecStartDate,
    CcRecEndDate,
    CcClosedDateSk,
    CcOpenDateSk,
    CcName,
    CcClass,
    CcEmployees,
    CcSqFt,
    CcHours,
    CcManager,
    CcMktId,
    CcMktClass,
    CcMktDesc,
    CcMarketManager,
    CcDivision,
    CcDivisionName,
    CcCompany,
    CcCompanyName,
    CcStreetNumber,
    CcStreetName,
    CcStreetType,
    CcSuiteNumber,
    CcCity,
    CcCounty,
    CcState,
    CcZip,
    CcCountry,
    CcGmtOffset,
    CcTaxPercentage,
}

impl CallCenterColumn {
    /// Get all columns in order
    pub fn values() -> &'static [CallCenterColumn] {
        use CallCenterColumn::*;
        static VALUES: &[CallCenterColumn] = &[
            CcCallCenterSk,
            CcCallCenterId,
            CcRecStartDate,
            CcRecEndDate,
            CcClosedDateSk,
            CcOpenDateSk,
            CcName,
            CcClass,
            CcEmployees,
            CcSqFt,
            CcHours,
            CcManager,
            CcMktId,
            CcMktClass,
            CcMktDesc,
            CcMarketManager,
            CcDivision,
            CcDivisionName,
            CcCompany,
            CcCompanyName,
            CcStreetNumber,
            CcStreetName,
            CcStreetType,
            CcSuiteNumber,
            CcCity,
            CcCounty,
            CcState,
            CcZip,
            CcCountry,
            CcGmtOffset,
            CcTaxPercentage,
        ];
        VALUES
    }

    /// Get the column type for this column
    fn get_column_type(&self) -> &'static ColumnType {
        use CallCenterColumn::*;
        match self {
            CcCallCenterSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            CcCallCenterId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(16))
            }
            CcRecStartDate => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::date().clone())
            }
            CcRecEndDate => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::date().clone())
            }
            CcClosedDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcOpenDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            CcClass => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            CcEmployees => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcSqFt => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcHours => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(20))
            }
            CcManager => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(40))
            }
            CcMktId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcMktClass => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(50))
            }
            CcMktDesc => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(100))
            }
            CcMarketManager => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(40))
            }
            CcDivision => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcDivisionName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(50))
            }
            CcCompany => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            CcCompanyName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(50))
            }
            CcStreetNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CcStreetName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            CcStreetType => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(15))
            }
            CcSuiteNumber => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CcCity => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(60))
            }
            CcCounty => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(30))
            }
            CcState => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(2))
            }
            CcZip => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(10))
            }
            CcCountry => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(20))
            }
            CcGmtOffset => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(5, 2))
            }
            CcTaxPercentage => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(5, 2))
            }
        }
    }
}

impl Column for CallCenterColumn {
    fn get_table(&self) -> Table {
        Table::CallCenter
    }

    fn get_name(&self) -> &'static str {
        use CallCenterColumn::*;
        match self {
            CcCallCenterSk => "cc_call_center_sk",
            CcCallCenterId => "cc_call_center_id",
            CcRecStartDate => "cc_rec_start_date",
            CcRecEndDate => "cc_rec_end_date",
            CcClosedDateSk => "cc_closed_date_sk",
            CcOpenDateSk => "cc_open_date_sk",
            CcName => "cc_name",
            CcClass => "cc_class",
            CcEmployees => "cc_employees",
            CcSqFt => "cc_sq_ft",
            CcHours => "cc_hours",
            CcManager => "cc_manager",
            CcMktId => "cc_mkt_id",
            CcMktClass => "cc_mkt_class",
            CcMktDesc => "cc_mkt_desc",
            CcMarketManager => "cc_market_manager",
            CcDivision => "cc_division",
            CcDivisionName => "cc_division_name",
            CcCompany => "cc_company",
            CcCompanyName => "cc_company_name",
            CcStreetNumber => "cc_street_number",
            CcStreetName => "cc_street_name",
            CcStreetType => "cc_street_type",
            CcSuiteNumber => "cc_suite_number",
            CcCity => "cc_city",
            CcCounty => "cc_county",
            CcState => "cc_state",
            CcZip => "cc_zip",
            CcCountry => "cc_country",
            CcGmtOffset => "cc_gmt_offset",
            CcTaxPercentage => "cc_tax_percentage",
        }
    }

    fn get_type(&self) -> &ColumnType {
        self.get_column_type()
    }

    fn get_position(&self) -> i32 {
        use CallCenterColumn::*;
        match self {
            CcCallCenterSk => 0,
            CcCallCenterId => 1,
            CcRecStartDate => 2,
            CcRecEndDate => 3,
            CcClosedDateSk => 4,
            CcOpenDateSk => 5,
            CcName => 6,
            CcClass => 7,
            CcEmployees => 8,
            CcSqFt => 9,
            CcHours => 10,
            CcManager => 11,
            CcMktId => 12,
            CcMktClass => 13,
            CcMktDesc => 14,
            CcMarketManager => 15,
            CcDivision => 16,
            CcDivisionName => 17,
            CcCompany => 18,
            CcCompanyName => 19,
            CcStreetNumber => 20,
            CcStreetName => 21,
            CcStreetType => 22,
            CcSuiteNumber => 23,
            CcCity => 24,
            CcCounty => 25,
            CcState => 26,
            CcZip => 27,
            CcCountry => 28,
            CcGmtOffset => 29,
            CcTaxPercentage => 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnTypeBase;

    #[test]
    fn test_call_center_column_basics() {
        let column = CallCenterColumn::CcCallCenterSk;
        assert_eq!(column.get_table(), Table::CallCenter);
        assert_eq!(column.get_name(), "cc_call_center_sk");
        assert_eq!(column.get_position(), 0);
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Identifier);
    }

    #[test]
    fn test_varchar_columns() {
        let column = CallCenterColumn::CcName;
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Varchar);
        assert_eq!(column.get_type().get_precision(), Some(50));
        assert_eq!(column.get_type().get_scale(), None);
        assert_eq!(column.get_type().get_sql_name(), "VARCHAR(50)");
    }

    #[test]
    fn test_char_columns() {
        let column = CallCenterColumn::CcCallCenterId;
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Char);
        assert_eq!(column.get_type().get_precision(), Some(16));
        assert_eq!(column.get_type().get_scale(), None);
        assert_eq!(column.get_type().get_sql_name(), "CHAR(16)");
    }

    #[test]
    fn test_decimal_columns() {
        let column = CallCenterColumn::CcGmtOffset;
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Decimal);
        assert_eq!(column.get_type().get_precision(), Some(5));
        assert_eq!(column.get_type().get_scale(), Some(2));
        assert_eq!(column.get_type().get_sql_name(), "DECIMAL(5,2)");
    }

    #[test]
    fn test_date_columns() {
        let column = CallCenterColumn::CcRecStartDate;
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Date);
        assert_eq!(column.get_type().get_precision(), None);
        assert_eq!(column.get_type().get_scale(), None);
        assert_eq!(column.get_type().get_sql_name(), "DATE");
    }

    #[test]
    fn test_integer_columns() {
        let column = CallCenterColumn::CcEmployees;
        assert_eq!(column.get_type().get_base(), ColumnTypeBase::Integer);
        assert_eq!(column.get_type().get_precision(), None);
        assert_eq!(column.get_type().get_scale(), None);
        assert_eq!(column.get_type().get_sql_name(), "INTEGER");
    }

    #[test]
    fn test_all_columns_count() {
        let columns = CallCenterColumn::values();
        assert_eq!(columns.len(), 31);
    }

    #[test]
    fn test_column_positions() {
        let columns = CallCenterColumn::values();
        for (index, column) in columns.iter().enumerate() {
            assert_eq!(column.get_position(), index as i32);
        }
    }

    #[test]
    fn test_column_names_lowercase() {
        for column in CallCenterColumn::values() {
            let name = column.get_name();
            assert_eq!(name, name.to_lowercase());
            assert!(name.starts_with("cc_"));
        }
    }
}
