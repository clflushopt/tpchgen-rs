use crate::column::{ColumnType, ColumnTypeBase};
use std::sync::OnceLock;

pub struct ColumnTypes;

impl ColumnTypes {
    pub fn integer() -> &'static ColumnType {
        static INTEGER: OnceLock<ColumnType> = OnceLock::new();
        INTEGER.get_or_init(|| ColumnType::simple(ColumnTypeBase::Integer))
    }

    /// IDENTIFIER type (for surrogate keys)
    pub fn identifier() -> &'static ColumnType {
        static IDENTIFIER: OnceLock<ColumnType> = OnceLock::new();
        IDENTIFIER.get_or_init(|| ColumnType::simple(ColumnTypeBase::Identifier))
    }

    /// DATE type
    pub fn date() -> &'static ColumnType {
        static DATE: OnceLock<ColumnType> = OnceLock::new();
        DATE.get_or_init(|| ColumnType::simple(ColumnTypeBase::Date))
    }

    /// TIME type
    pub fn time() -> &'static ColumnType {
        static TIME: OnceLock<ColumnType> = OnceLock::new();
        TIME.get_or_init(|| ColumnType::simple(ColumnTypeBase::Time))
    }

    /// VARCHAR type with specified precision
    pub fn varchar(precision: i32) -> ColumnType {
        ColumnType::with_precision(ColumnTypeBase::Varchar, precision)
            .expect("VARCHAR type creation should not fail")
    }

    /// CHAR type with specified precision  
    pub fn character(precision: i32) -> ColumnType {
        ColumnType::with_precision(ColumnTypeBase::Char, precision)
            .expect("CHAR type creation should not fail")
    }

    /// DECIMAL type with specified precision and scale
    pub fn decimal(precision: i32, scale: i32) -> ColumnType {
        ColumnType::with_precision_and_scale(ColumnTypeBase::Decimal, precision, scale)
            .expect("DECIMAL type creation should not fail")
    }

    /// Create VARCHAR type (convenience alias)
    pub fn var_char(precision: i32) -> ColumnType {
        Self::varchar(precision)
    }

    /// Create CHAR type (convenience alias)
    pub fn char(precision: i32) -> ColumnType {
        Self::character(precision)
    }

    /// Get all standard types for testing/introspection
    pub fn standard_types() -> Vec<&'static ColumnType> {
        vec![
            Self::integer(),
            Self::identifier(),
            Self::date(),
            Self::time(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnTypeBase;

    #[test]
    fn test_standard_types() {
        assert_eq!(ColumnTypes::integer().get_base(), ColumnTypeBase::Integer);
        assert_eq!(
            ColumnTypes::identifier().get_base(),
            ColumnTypeBase::Identifier
        );
        assert_eq!(ColumnTypes::date().get_base(), ColumnTypeBase::Date);
        assert_eq!(ColumnTypes::time().get_base(), ColumnTypeBase::Time);
    }

    #[test]
    fn test_varchar_creation() {
        let varchar50 = ColumnTypes::varchar(50);
        assert_eq!(varchar50.get_base(), ColumnTypeBase::Varchar);
        assert_eq!(varchar50.get_precision(), Some(50));
        assert_eq!(varchar50.get_scale(), None);
        assert_eq!(varchar50.get_sql_name(), "VARCHAR(50)");

        let varchar100 = ColumnTypes::var_char(100);
        assert_eq!(varchar100.get_precision(), Some(100));
    }

    #[test]
    fn test_character_creation() {
        let char10 = ColumnTypes::character(10);
        assert_eq!(char10.get_base(), ColumnTypeBase::Char);
        assert_eq!(char10.get_precision(), Some(10));
        assert_eq!(char10.get_scale(), None);
        assert_eq!(char10.get_sql_name(), "CHAR(10)");

        let char20 = ColumnTypes::char(20);
        assert_eq!(char20.get_precision(), Some(20));
    }

    #[test]
    fn test_decimal_creation() {
        let decimal10_2 = ColumnTypes::decimal(10, 2);
        assert_eq!(decimal10_2.get_base(), ColumnTypeBase::Decimal);
        assert_eq!(decimal10_2.get_precision(), Some(10));
        assert_eq!(decimal10_2.get_scale(), Some(2));
        assert_eq!(decimal10_2.get_sql_name(), "DECIMAL(10,2)");
    }

    #[test]
    fn test_singleton_behavior() {
        let int1 = ColumnTypes::integer();
        let int2 = ColumnTypes::integer();
        assert!(std::ptr::eq(int1, int2));

        let date1 = ColumnTypes::date();
        let date2 = ColumnTypes::date();
        assert!(std::ptr::eq(date1, date2));
    }

    #[test]
    fn test_standard_types_collection() {
        let types = ColumnTypes::standard_types();
        assert_eq!(types.len(), 4);

        // Verify each type is present
        assert!(types
            .iter()
            .any(|t| t.get_base() == ColumnTypeBase::Integer));
        assert!(types
            .iter()
            .any(|t| t.get_base() == ColumnTypeBase::Identifier));
        assert!(types.iter().any(|t| t.get_base() == ColumnTypeBase::Date));
        assert!(types.iter().any(|t| t.get_base() == ColumnTypeBase::Time));
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", ColumnTypes::integer()), "INTEGER");
        assert_eq!(format!("{}", ColumnTypes::varchar(255)), "VARCHAR(255)");
        assert_eq!(format!("{}", ColumnTypes::decimal(18, 6)), "DECIMAL(18,6)");
    }

    #[test]
    fn test_various_precisions() {
        // Test different VARCHAR sizes common in TPC-DS
        let small_varchar = ColumnTypes::varchar(16);
        let medium_varchar = ColumnTypes::varchar(50);
        let large_varchar = ColumnTypes::varchar(100);

        assert_eq!(small_varchar.get_precision(), Some(16));
        assert_eq!(medium_varchar.get_precision(), Some(50));
        assert_eq!(large_varchar.get_precision(), Some(100));

        // Test different CHAR sizes
        let small_char = ColumnTypes::character(1);
        let medium_char = ColumnTypes::character(10);
        let large_char = ColumnTypes::character(50);

        assert_eq!(small_char.get_precision(), Some(1));
        assert_eq!(medium_char.get_precision(), Some(10));
        assert_eq!(large_char.get_precision(), Some(50));

        // Test different DECIMAL precisions/scales
        let currency_decimal = ColumnTypes::decimal(7, 2); // Common for prices
        let percentage_decimal = ColumnTypes::decimal(5, 4); // Common for percentages
        let large_decimal = ColumnTypes::decimal(15, 2); // Common for large amounts

        assert_eq!(currency_decimal.get_precision(), Some(7));
        assert_eq!(currency_decimal.get_scale(), Some(2));
        assert_eq!(percentage_decimal.get_precision(), Some(5));
        assert_eq!(percentage_decimal.get_scale(), Some(4));
        assert_eq!(large_decimal.get_precision(), Some(15));
        assert_eq!(large_decimal.get_scale(), Some(2));
    }
}
