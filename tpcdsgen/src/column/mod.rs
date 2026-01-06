pub mod call_center;
pub mod column_type;
pub mod column_types;
pub mod customer_address_column;
pub mod customer_column;
pub mod dbgen_version;
pub mod household_demographics;
pub mod inventory_column;
pub mod promotion;
pub mod web_site;

pub use call_center::CallCenterColumn;
pub use column_type::{ColumnType, ColumnTypeBase};
pub use column_types::ColumnTypes;
pub use customer_address_column::CustomerAddressColumn;
pub use customer_column::CustomerColumn;
pub use dbgen_version::DbgenVersionColumn;
pub use household_demographics::HouseholdDemographicsColumn;
pub use inventory_column::InventoryColumn;
pub use promotion::PromotionColumn;
pub use web_site::WebSiteColumn;

// Re-export Table from crate::table to provide a single source of truth
// This eliminates the duplicate Table enum that previously existed here
pub use crate::table::Table;

/// Column trait for TPC-DS table columns.
/// Used by column enums (CallCenterColumn, CustomerColumn, etc.) to provide
/// common functionality for column metadata access.
pub trait Column: Send + Sync {
    /// Get the table this column belongs to
    fn get_table(&self) -> Table;

    /// Get the column name (lowercase)
    fn get_name(&self) -> &'static str;

    /// Get the column type
    fn get_type(&self) -> &ColumnType;

    /// Get the column position (0-based ordinal)
    fn get_position(&self) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_name() {
        assert_eq!(Table::CallCenter.get_name(), "call_center");
        assert_eq!(format!("{}", Table::CallCenter), "call_center");
    }
}
