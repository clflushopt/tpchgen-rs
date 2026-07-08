use crate::row::table_row::{dat_field, dat_zip, DatField};
use crate::row::TableRow;
use crate::types::Address;
use std::fmt;

/// Warehouse table row (WarehouseRow)
#[derive(Debug, Clone)]
pub struct WarehouseRow {
    null_bit_map: i64,
    w_warehouse_sk: i64,
    w_warehouse_id: String,
    w_warehouse_name: String,
    w_warehouse_sq_ft: i32,
    w_address: Address,
}

impl WarehouseRow {
    pub fn new(
        null_bit_map: i64,
        w_warehouse_sk: i64,
        w_warehouse_id: String,
        w_warehouse_name: String,
        w_warehouse_sq_ft: i32,
        w_address: Address,
    ) -> Self {
        WarehouseRow {
            null_bit_map,
            w_warehouse_sk,
            w_warehouse_id,
            w_warehouse_name,
            w_warehouse_sq_ft,
            w_address,
        }
    }

    /// Check if a column should be null based on the null bitmap (TableRowWithNulls logic)
    fn should_be_null(&self, column_position: i32) -> bool {
        ((self.null_bit_map >> column_position) & 1) == 1
    }

    /// Convert value to string or empty string if null (getStringOrNull)
    fn get_string_or_null<T: ToString>(&self, value: T, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Convert key to string or empty string if null (getStringOrNullForKey)
    fn get_string_or_null_for_key(&self, value: i64, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            value.to_string()
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_w_warehouse_sk(&self) -> i64 {
        self.w_warehouse_sk
    }

    pub fn get_w_warehouse_id(&self) -> &str {
        &self.w_warehouse_id
    }

    pub fn get_w_warehouse_name(&self) -> &str {
        &self.w_warehouse_name
    }

    pub fn get_w_warehouse_sq_ft(&self) -> i32 {
        self.w_warehouse_sq_ft
    }

    pub fn get_w_address(&self) -> &Address {
        &self.w_address
    }
}

/// DAT field helper mirroring `get_string_or_null`/`get_string_or_null_for_key`
/// (warehouse applies no key sentinel check, only the null bit).
impl WarehouseRow {
    fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        dat_field(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for WarehouseRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.w_warehouse_sk, 0),
            self.field(&self.w_warehouse_id, 1),
            self.field(&self.w_warehouse_name, 2),
            self.field(self.w_warehouse_sq_ft, 3),
            self.field(self.w_address.get_street_number(), 4),
            self.field(self.w_address.get_street_name(), 5),
            self.field(self.w_address.get_street_type(), 6),
            self.field(self.w_address.get_suite_number(), 7),
            self.field(self.w_address.get_city(), 8),
            self.field(self.w_address.get_county().unwrap_or(""), 9),
            self.field(self.w_address.get_state(), 10),
            dat_zip(self.w_address.get_zip(), self.should_be_null(11)),
            self.field(self.w_address.get_country(), 12),
            self.field(self.w_address.get_gmt_offset(), 13),
        )
    }
}

impl TableRow for WarehouseRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java WarehouseGeneratorColumn
        // First column (W_WAREHOUSE_SK) is at global position 351, so relative positions are 0-13
        vec![
            self.get_string_or_null_for_key(self.w_warehouse_sk, 0),
            self.get_string_or_null(&self.w_warehouse_id, 1),
            self.get_string_or_null(&self.w_warehouse_name, 2),
            self.get_string_or_null(self.w_warehouse_sq_ft, 3),
            self.get_string_or_null(self.w_address.get_street_number(), 4),
            self.get_string_or_null(self.w_address.get_street_name(), 5),
            self.get_string_or_null(self.w_address.get_street_type(), 6),
            self.get_string_or_null(self.w_address.get_suite_number(), 7),
            self.get_string_or_null(self.w_address.get_city(), 8),
            self.get_string_or_null(self.w_address.get_county().unwrap_or(""), 9),
            self.get_string_or_null(self.w_address.get_state(), 10),
            self.get_string_or_null(format!("{:05}", self.w_address.get_zip()), 11),
            self.get_string_or_null(self.w_address.get_country(), 12),
            self.get_string_or_null(self.w_address.get_gmt_offset(), 13),
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
        let row = WarehouseRow::new(
            0b10,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            "Conventional childr".to_string(),
            977787,
            test_address(),
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
