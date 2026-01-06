use crate::row::TableRow;

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
