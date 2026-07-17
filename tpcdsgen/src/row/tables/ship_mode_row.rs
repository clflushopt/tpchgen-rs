use crate::row::table_row::DatField;
use crate::row::TableRow;
use std::fmt;

/// Ship mode table row (ShipModeRow)
#[derive(Debug, Clone)]
pub struct ShipModeRow {
    null_bit_map: i64,
    sm_ship_mode_sk: i64,
    sm_ship_mode_id: String,
    sm_type: String,
    sm_code: String,
    sm_carrier: String,
    sm_contract: String,
}

impl ShipModeRow {
    pub fn new(
        null_bit_map: i64,
        sm_ship_mode_sk: i64,
        sm_ship_mode_id: String,
        sm_type: String,
        sm_code: String,
        sm_carrier: String,
        sm_contract: String,
    ) -> Self {
        ShipModeRow {
            null_bit_map,
            sm_ship_mode_sk,
            sm_ship_mode_id,
            sm_type,
            sm_code,
            sm_carrier,
            sm_contract,
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

    pub fn get_sm_ship_mode_sk(&self) -> i64 {
        self.sm_ship_mode_sk
    }

    pub fn get_sm_ship_mode_id(&self) -> &str {
        &self.sm_ship_mode_id
    }

    pub fn get_sm_type(&self) -> &str {
        &self.sm_type
    }

    pub fn get_sm_code(&self) -> &str {
        &self.sm_code
    }

    pub fn get_sm_carrier(&self) -> &str {
        &self.sm_carrier
    }

    pub fn get_sm_contract(&self) -> &str {
        &self.sm_contract
    }
}

/// DAT field helper mirroring `get_string_or_null`/`get_string_or_null_for_key`
/// (ship_mode applies no key sentinel check, only the null bit).
impl ShipModeRow {
    fn field<T>(&self, value: T, column_position: i32) -> DatField<T> {
        DatField::new(value, self.should_be_null(column_position))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for ShipModeRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|",
            self.field(self.sm_ship_mode_sk, 0),
            self.field(&self.sm_ship_mode_id, 1),
            self.field(&self.sm_type, 2),
            self.field(&self.sm_code, 3),
            self.field(&self.sm_carrier, 4),
            self.field(&self.sm_contract, 5),
        )
    }
}

impl TableRow for ShipModeRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java ShipModeGeneratorColumn
        // First column (SM_SHIP_MODE_SK) is at global position 252, so relative positions are 0-5
        vec![
            self.get_string_or_null_for_key(self.sm_ship_mode_sk, 0),
            self.get_string_or_null(&self.sm_ship_mode_id, 1),
            self.get_string_or_null(&self.sm_type, 2),
            self.get_string_or_null(&self.sm_code, 3),
            self.get_string_or_null(&self.sm_carrier, 4),
            self.get_string_or_null(&self.sm_contract, 5),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_matches_get_values() {
        let row = ShipModeRow::new(
            0b10,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            "EXPRESS".to_string(),
            "AIR".to_string(),
            "UPS".to_string(),
            "2mM8tgcDE0aNiHg5heb".to_string(),
        );
        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
