use crate::{check_state, error::Result, TpcdsError};

/// SQL column type base enumeration (ColumnType.Base)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnTypeBase {
    Integer,
    Identifier,
    Date,
    Decimal,
    Varchar,
    Char,
    Time,
}

/// SQL column type with optional precision and scale (ColumnType)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnType {
    base: ColumnTypeBase,
    precision: Option<i32>,
    scale: Option<i32>,
}

impl ColumnType {
    /// Create a new column type with base, precision, and scale
    pub fn new(base: ColumnTypeBase, precision: Option<i32>, scale: Option<i32>) -> Result<Self> {
        // Validation matching Java implementation
        if base == ColumnTypeBase::Varchar {
            check_state!(precision.is_some(), "VARCHAR must have precision");
        }
        if base == ColumnTypeBase::Decimal {
            check_state!(precision.is_some(), "DECIMAL must have precision");
            check_state!(scale.is_some(), "DECIMAL must have scale");
        }

        Ok(ColumnType {
            base,
            precision,
            scale,
        })
    }

    /// Create column type with precision and scale
    pub fn with_precision_and_scale(
        base: ColumnTypeBase,
        precision: i32,
        scale: i32,
    ) -> Result<Self> {
        Self::new(base, Some(precision), Some(scale))
    }

    /// Create column type with precision only
    pub fn with_precision(base: ColumnTypeBase, precision: i32) -> Result<Self> {
        Self::new(base, Some(precision), None)
    }

    /// Create column type with no precision or scale
    pub fn simple(base: ColumnTypeBase) -> Self {
        ColumnType {
            base,
            precision: None,
            scale: None,
        }
    }

    /// Get the base type
    pub fn get_base(&self) -> ColumnTypeBase {
        self.base
    }

    /// Get the precision (if any)
    pub fn get_precision(&self) -> Option<i32> {
        self.precision
    }

    /// Get the scale (if any)
    pub fn get_scale(&self) -> Option<i32> {
        self.scale
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self.base,
            ColumnTypeBase::Integer | ColumnTypeBase::Identifier | ColumnTypeBase::Decimal
        )
    }

    /// Check if this is a string type
    pub fn is_string(&self) -> bool {
        matches!(self.base, ColumnTypeBase::Varchar | ColumnTypeBase::Char)
    }

    /// Check if this is a temporal type
    pub fn is_temporal(&self) -> bool {
        matches!(self.base, ColumnTypeBase::Date | ColumnTypeBase::Time)
    }

    /// Get SQL type name for display purposes
    pub fn get_sql_name(&self) -> String {
        match self.base {
            ColumnTypeBase::Integer => "INTEGER".to_string(),
            ColumnTypeBase::Identifier => "IDENTIFIER".to_string(),
            ColumnTypeBase::Date => "DATE".to_string(),
            ColumnTypeBase::Time => "TIME".to_string(),
            ColumnTypeBase::Varchar => {
                if let Some(precision) = self.precision {
                    format!("VARCHAR({})", precision)
                } else {
                    "VARCHAR".to_string()
                }
            }
            ColumnTypeBase::Char => {
                if let Some(precision) = self.precision {
                    format!("CHAR({})", precision)
                } else {
                    "CHAR".to_string()
                }
            }
            ColumnTypeBase::Decimal => match (self.precision, self.scale) {
                (Some(p), Some(s)) => format!("DECIMAL({},{})", p, s),
                (Some(p), None) => format!("DECIMAL({})", p),
                _ => "DECIMAL".to_string(),
            },
        }
    }
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_sql_name())
    }
}

impl std::fmt::Display for ColumnTypeBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ColumnTypeBase::Integer => "INTEGER",
            ColumnTypeBase::Identifier => "IDENTIFIER",
            ColumnTypeBase::Date => "DATE",
            ColumnTypeBase::Decimal => "DECIMAL",
            ColumnTypeBase::Varchar => "VARCHAR",
            ColumnTypeBase::Char => "CHAR",
            ColumnTypeBase::Time => "TIME",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_types() {
        let integer_type = ColumnType::simple(ColumnTypeBase::Integer);
        assert_eq!(integer_type.get_base(), ColumnTypeBase::Integer);
        assert_eq!(integer_type.get_precision(), None);
        assert_eq!(integer_type.get_scale(), None);
        assert_eq!(integer_type.get_sql_name(), "INTEGER");

        let date_type = ColumnType::simple(ColumnTypeBase::Date);
        assert_eq!(date_type.get_base(), ColumnTypeBase::Date);
        assert_eq!(date_type.get_sql_name(), "DATE");
    }

    #[test]
    fn test_varchar_with_precision() {
        let varchar_type = ColumnType::with_precision(ColumnTypeBase::Varchar, 50).unwrap();
        assert_eq!(varchar_type.get_base(), ColumnTypeBase::Varchar);
        assert_eq!(varchar_type.get_precision(), Some(50));
        assert_eq!(varchar_type.get_scale(), None);
        assert_eq!(varchar_type.get_sql_name(), "VARCHAR(50)");
    }

    #[test]
    fn test_char_with_precision() {
        let char_type = ColumnType::with_precision(ColumnTypeBase::Char, 10).unwrap();
        assert_eq!(char_type.get_base(), ColumnTypeBase::Char);
        assert_eq!(char_type.get_precision(), Some(10));
        assert_eq!(char_type.get_sql_name(), "CHAR(10)");
    }

    #[test]
    fn test_decimal_with_precision_and_scale() {
        let decimal_type =
            ColumnType::with_precision_and_scale(ColumnTypeBase::Decimal, 10, 2).unwrap();
        assert_eq!(decimal_type.get_base(), ColumnTypeBase::Decimal);
        assert_eq!(decimal_type.get_precision(), Some(10));
        assert_eq!(decimal_type.get_scale(), Some(2));
        assert_eq!(decimal_type.get_sql_name(), "DECIMAL(10,2)");
    }

    #[test]
    fn test_varchar_validation() {
        // VARCHAR must have precision
        assert!(ColumnType::new(ColumnTypeBase::Varchar, None, None).is_err());
        assert!(ColumnType::new(ColumnTypeBase::Varchar, Some(50), None).is_ok());
    }

    #[test]
    fn test_decimal_validation() {
        // DECIMAL must have both precision and scale
        assert!(ColumnType::new(ColumnTypeBase::Decimal, None, None).is_err());
        assert!(ColumnType::new(ColumnTypeBase::Decimal, Some(10), None).is_err());
        assert!(ColumnType::new(ColumnTypeBase::Decimal, None, Some(2)).is_err());
        assert!(ColumnType::new(ColumnTypeBase::Decimal, Some(10), Some(2)).is_ok());
    }

    #[test]
    fn test_type_classification() {
        let integer_type = ColumnType::simple(ColumnTypeBase::Integer);
        assert!(integer_type.is_numeric());
        assert!(!integer_type.is_string());
        assert!(!integer_type.is_temporal());

        let varchar_type = ColumnType::with_precision(ColumnTypeBase::Varchar, 50).unwrap();
        assert!(!varchar_type.is_numeric());
        assert!(varchar_type.is_string());
        assert!(!varchar_type.is_temporal());

        let date_type = ColumnType::simple(ColumnTypeBase::Date);
        assert!(!date_type.is_numeric());
        assert!(!date_type.is_string());
        assert!(date_type.is_temporal());

        let decimal_type =
            ColumnType::with_precision_and_scale(ColumnTypeBase::Decimal, 10, 2).unwrap();
        assert!(decimal_type.is_numeric());
        assert!(!decimal_type.is_string());
        assert!(!decimal_type.is_temporal());
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", ColumnType::simple(ColumnTypeBase::Integer)),
            "INTEGER"
        );
        assert_eq!(
            format!(
                "{}",
                ColumnType::with_precision(ColumnTypeBase::Varchar, 100).unwrap()
            ),
            "VARCHAR(100)"
        );
        assert_eq!(
            format!(
                "{}",
                ColumnType::with_precision_and_scale(ColumnTypeBase::Decimal, 15, 4).unwrap()
            ),
            "DECIMAL(15,4)"
        );
    }

    #[test]
    fn test_equality() {
        let type1 = ColumnType::with_precision(ColumnTypeBase::Varchar, 50).unwrap();
        let type2 = ColumnType::with_precision(ColumnTypeBase::Varchar, 50).unwrap();
        let type3 = ColumnType::with_precision(ColumnTypeBase::Varchar, 100).unwrap();

        assert_eq!(type1, type2);
        assert_ne!(type1, type3);
    }
}
