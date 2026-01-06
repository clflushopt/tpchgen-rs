use crate::row::TableRow;

/// Reason table row (ReasonRow)
#[derive(Debug, Clone)]
pub struct ReasonRow {
    null_bit_map: i64,
    r_reason_sk: i64,
    r_reason_id: String,
    r_reason_description: String,
}

impl ReasonRow {
    pub fn new(
        null_bit_map: i64,
        r_reason_sk: i64,
        r_reason_id: String,
        r_reason_description: String,
    ) -> Self {
        ReasonRow {
            null_bit_map,
            r_reason_sk,
            r_reason_id,
            r_reason_description,
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

    pub fn get_r_reason_sk(&self) -> i64 {
        self.r_reason_sk
    }

    pub fn get_r_reason_id(&self) -> &str {
        &self.r_reason_id
    }

    pub fn get_r_reason_description(&self) -> &str {
        &self.r_reason_description
    }
}

impl TableRow for ReasonRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java ReasonGeneratorColumn
        // First column (R_REASON_SK) is at global position 248, so relative positions are 0-2
        vec![
            self.get_string_or_null_for_key(self.r_reason_sk, 0),
            self.get_string_or_null(&self.r_reason_id, 1),
            self.get_string_or_null(&self.r_reason_description, 2),
        ]
    }
}
