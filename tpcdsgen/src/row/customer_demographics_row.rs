use crate::row::TableRow;

/// Customer demographics table row (CustomerDemographicsRow)
#[derive(Debug, Clone)]
pub struct CustomerDemographicsRow {
    null_bit_map: i64,
    cd_demo_sk: i64,
    cd_gender: String,
    cd_marital_status: String,
    cd_education_status: String,
    cd_purchase_estimate: i32,
    cd_credit_rating: String,
    cd_dep_count: i32,
    cd_dep_employed_count: i32,
    cd_dep_college_count: i32,
}

impl CustomerDemographicsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cd_demo_sk: i64,
        cd_gender: String,
        cd_marital_status: String,
        cd_education_status: String,
        cd_purchase_estimate: i32,
        cd_credit_rating: String,
        cd_dep_count: i32,
        cd_dep_employed_count: i32,
        cd_dep_college_count: i32,
    ) -> Self {
        CustomerDemographicsRow {
            null_bit_map,
            cd_demo_sk,
            cd_gender,
            cd_marital_status,
            cd_education_status,
            cd_purchase_estimate,
            cd_credit_rating,
            cd_dep_count,
            cd_dep_employed_count,
            cd_dep_college_count,
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
}

impl TableRow for CustomerDemographicsRow {
    fn get_values(&self) -> Vec<String> {
        // Column positions match Java CustomerDemographicsGeneratorColumn (0-8)
        vec![
            self.get_string_or_null(self.cd_demo_sk, 0),
            self.get_string_or_null(&self.cd_gender, 1),
            self.get_string_or_null(&self.cd_marital_status, 2),
            self.get_string_or_null(&self.cd_education_status, 3),
            self.get_string_or_null(self.cd_purchase_estimate, 4),
            self.get_string_or_null(&self.cd_credit_rating, 5),
            self.get_string_or_null(self.cd_dep_count, 6),
            self.get_string_or_null(self.cd_dep_employed_count, 7),
            self.get_string_or_null(self.cd_dep_college_count, 8),
        ]
    }
}
