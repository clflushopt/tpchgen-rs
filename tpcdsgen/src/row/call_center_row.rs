use crate::row::TableRow;
use crate::types::{Address, Decimal};

/// Call Center row data structure (CallCenterRow)
/// Contains all fields for the CALL_CENTER table in TPC-DS
#[derive(Debug, Clone, PartialEq)]
pub struct CallCenterRow {
    // Primary key
    cc_call_center_sk: i64,

    // Business key and versioning
    cc_call_center_id: String,
    cc_rec_start_date_id: String,
    cc_rec_end_date_id: String,
    cc_closed_date_id: String,
    cc_open_date_id: String,

    // Call center information
    cc_name: String,
    cc_class: String,
    cc_employees: i32,
    cc_sq_ft: i32,
    cc_hours: String,
    cc_manager: String,

    // Market information
    cc_market_id: i32,
    cc_market_class: String,
    cc_market_desc: String,
    cc_market_manager: String,

    // Organization hierarchy
    cc_division_id: i32,
    cc_division_name: String,
    cc_company: i32,
    cc_company_name: String,

    // Address information (embedded)
    cc_address: Address,

    // Financial information
    cc_tax_percentage: Decimal,

    // Null bitmap for handling null values
    null_bit_map: i64,
}

impl CallCenterRow {
    /// Create a new builder for CallCenterRow
    pub fn builder() -> CallCenterRowBuilder {
        CallCenterRowBuilder::new()
    }

    // Getter methods (matching Java implementation)
    pub fn get_cc_call_center_sk(&self) -> i64 {
        self.cc_call_center_sk
    }

    pub fn get_cc_call_center_id(&self) -> &str {
        &self.cc_call_center_id
    }

    pub fn get_cc_rec_start_date_id(&self) -> &str {
        &self.cc_rec_start_date_id
    }

    pub fn get_cc_rec_end_date_id(&self) -> &str {
        &self.cc_rec_end_date_id
    }

    pub fn get_cc_closed_date_id(&self) -> &str {
        &self.cc_closed_date_id
    }

    pub fn get_cc_open_date_id(&self) -> &str {
        &self.cc_open_date_id
    }

    pub fn get_cc_name(&self) -> &str {
        &self.cc_name
    }

    pub fn get_cc_class(&self) -> &str {
        &self.cc_class
    }

    pub fn get_cc_employees(&self) -> i32 {
        self.cc_employees
    }

    pub fn get_cc_sq_ft(&self) -> i32 {
        self.cc_sq_ft
    }

    pub fn get_cc_hours(&self) -> &str {
        &self.cc_hours
    }

    pub fn get_cc_manager(&self) -> &str {
        &self.cc_manager
    }

    pub fn get_cc_market_id(&self) -> i32 {
        self.cc_market_id
    }

    pub fn get_cc_market_class(&self) -> &str {
        &self.cc_market_class
    }

    pub fn get_cc_market_desc(&self) -> &str {
        &self.cc_market_desc
    }

    pub fn get_cc_market_manager(&self) -> &str {
        &self.cc_market_manager
    }

    pub fn get_cc_division_id(&self) -> i32 {
        self.cc_division_id
    }

    pub fn get_cc_division_name(&self) -> &str {
        &self.cc_division_name
    }

    pub fn get_cc_company(&self) -> i32 {
        self.cc_company
    }

    pub fn get_cc_company_name(&self) -> &str {
        &self.cc_company_name
    }

    pub fn get_cc_address(&self) -> &Address {
        &self.cc_address
    }

    pub fn get_cc_tax_percentage(&self) -> &Decimal {
        &self.cc_tax_percentage
    }

    pub fn get_null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    /// Check if a field should be null based on the null bitmap
    fn is_null(&self, column_position: i32) -> bool {
        (self.null_bit_map & (1 << column_position)) != 0
    }

    /// Format a value as string, handling nulls
    fn format_value(&self, value: &str, column_position: i32) -> String {
        if self.is_null(column_position) {
            "NULL".to_string()
        } else {
            value.to_string()
        }
    }

    /// Format a numeric value as string, handling nulls
    fn format_numeric<T: std::fmt::Display>(&self, value: T, column_position: i32) -> String {
        if self.is_null(column_position) {
            "NULL".to_string()
        } else {
            value.to_string()
        }
    }
}

impl TableRow for CallCenterRow {
    /// Get all values as strings for CSV output (getValues())
    fn get_values(&self) -> Vec<String> {
        vec![
            self.format_numeric(self.cc_call_center_sk, 0),
            self.format_value(&self.cc_call_center_id, 1),
            self.format_value(&self.cc_rec_start_date_id, 2),
            self.format_value(&self.cc_rec_end_date_id, 3),
            self.format_value(&self.cc_closed_date_id, 4),
            self.format_value(&self.cc_open_date_id, 5),
            self.format_value(&self.cc_name, 6),
            self.format_value(&self.cc_class, 7),
            self.format_numeric(self.cc_employees, 8),
            self.format_numeric(self.cc_sq_ft, 9),
            self.format_value(&self.cc_hours, 10),
            self.format_value(&self.cc_manager, 11),
            self.format_numeric(self.cc_market_id, 12),
            self.format_value(&self.cc_market_class, 13),
            self.format_value(&self.cc_market_desc, 14),
            self.format_value(&self.cc_market_manager, 15),
            self.format_numeric(self.cc_division_id, 16),
            self.format_value(&self.cc_division_name, 17),
            self.format_numeric(self.cc_company, 18),
            self.format_value(&self.cc_company_name, 19),
            // Address fields (flattened)
            self.format_numeric(self.cc_address.get_street_number(), 20),
            self.format_value(&self.cc_address.get_street_name(), 21),
            self.format_value(self.cc_address.get_street_type(), 22),
            self.format_value(self.cc_address.get_suite_number(), 23),
            self.format_value(self.cc_address.get_city(), 24),
            self.format_value(self.cc_address.get_county().unwrap_or(""), 25),
            self.format_value(self.cc_address.get_state(), 26),
            self.format_numeric(self.cc_address.get_zip(), 27),
            self.format_value(self.cc_address.get_country(), 28),
            self.format_numeric(self.cc_address.get_gmt_offset(), 29),
            self.format_value(&self.cc_tax_percentage.to_string(), 30),
        ]
    }
}

/// Builder for CallCenterRow (CallCenterRow.Builder)
#[derive(Debug, Default)]
pub struct CallCenterRowBuilder {
    cc_call_center_sk: Option<i64>,
    cc_call_center_id: Option<String>,
    cc_rec_start_date_id: Option<String>,
    cc_rec_end_date_id: Option<String>,
    cc_closed_date_id: Option<String>,
    cc_open_date_id: Option<String>,
    cc_name: Option<String>,
    cc_class: Option<String>,
    cc_employees: Option<i32>,
    cc_sq_ft: Option<i32>,
    cc_hours: Option<String>,
    cc_manager: Option<String>,
    cc_market_id: Option<i32>,
    cc_market_class: Option<String>,
    cc_market_desc: Option<String>,
    cc_market_manager: Option<String>,
    cc_division_id: Option<i32>,
    cc_division_name: Option<String>,
    cc_company: Option<i32>,
    cc_company_name: Option<String>,
    cc_address: Option<Address>,
    cc_tax_percentage: Option<Decimal>,
    null_bit_map: Option<i64>,
}

impl CallCenterRowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // Setter methods (matching Java builder pattern)
    pub fn set_cc_call_center_sk(mut self, value: i64) -> Self {
        self.cc_call_center_sk = Some(value);
        self
    }

    pub fn set_cc_call_center_id(mut self, value: String) -> Self {
        self.cc_call_center_id = Some(value);
        self
    }

    pub fn set_cc_rec_start_date_id(mut self, value: String) -> Self {
        self.cc_rec_start_date_id = Some(value);
        self
    }

    pub fn set_cc_rec_end_date_id(mut self, value: String) -> Self {
        self.cc_rec_end_date_id = Some(value);
        self
    }

    pub fn set_cc_closed_date_id(mut self, value: String) -> Self {
        self.cc_closed_date_id = Some(value);
        self
    }

    pub fn set_cc_open_date_id(mut self, value: String) -> Self {
        self.cc_open_date_id = Some(value);
        self
    }

    pub fn set_cc_name(mut self, value: String) -> Self {
        self.cc_name = Some(value);
        self
    }

    pub fn set_cc_class(mut self, value: String) -> Self {
        self.cc_class = Some(value);
        self
    }

    pub fn set_cc_employees(mut self, value: i32) -> Self {
        self.cc_employees = Some(value);
        self
    }

    pub fn set_cc_sq_ft(mut self, value: i32) -> Self {
        self.cc_sq_ft = Some(value);
        self
    }

    pub fn set_cc_hours(mut self, value: String) -> Self {
        self.cc_hours = Some(value);
        self
    }

    pub fn set_cc_manager(mut self, value: String) -> Self {
        self.cc_manager = Some(value);
        self
    }

    pub fn set_cc_market_id(mut self, value: i32) -> Self {
        self.cc_market_id = Some(value);
        self
    }

    pub fn set_cc_market_class(mut self, value: String) -> Self {
        self.cc_market_class = Some(value);
        self
    }

    pub fn set_cc_market_desc(mut self, value: String) -> Self {
        self.cc_market_desc = Some(value);
        self
    }

    pub fn set_cc_market_manager(mut self, value: String) -> Self {
        self.cc_market_manager = Some(value);
        self
    }

    pub fn set_cc_division_id(mut self, value: i32) -> Self {
        self.cc_division_id = Some(value);
        self
    }

    pub fn set_cc_division_name(mut self, value: String) -> Self {
        self.cc_division_name = Some(value);
        self
    }

    pub fn set_cc_company(mut self, value: i32) -> Self {
        self.cc_company = Some(value);
        self
    }

    pub fn set_cc_company_name(mut self, value: String) -> Self {
        self.cc_company_name = Some(value);
        self
    }

    pub fn set_cc_address(mut self, value: Address) -> Self {
        self.cc_address = Some(value);
        self
    }

    pub fn set_cc_tax_percentage(mut self, value: Decimal) -> Self {
        self.cc_tax_percentage = Some(value);
        self
    }

    pub fn set_null_bit_map(mut self, value: i64) -> Self {
        self.null_bit_map = Some(value);
        self
    }

    /// Build the CallCenterRow
    pub fn build(self) -> CallCenterRow {
        CallCenterRow {
            cc_call_center_sk: self.cc_call_center_sk.unwrap_or(0),
            cc_call_center_id: self.cc_call_center_id.unwrap_or_default(),
            cc_rec_start_date_id: self.cc_rec_start_date_id.unwrap_or_default(),
            cc_rec_end_date_id: self.cc_rec_end_date_id.unwrap_or_default(),
            cc_closed_date_id: self.cc_closed_date_id.unwrap_or_default(), // Default empty for null
            cc_open_date_id: self.cc_open_date_id.unwrap_or_default(),
            cc_name: self.cc_name.unwrap_or_default(),
            cc_class: self.cc_class.unwrap_or_default(),
            cc_employees: self.cc_employees.unwrap_or(0),
            cc_sq_ft: self.cc_sq_ft.unwrap_or(0),
            cc_hours: self.cc_hours.unwrap_or_default(),
            cc_manager: self.cc_manager.unwrap_or_default(),
            cc_market_id: self.cc_market_id.unwrap_or(0),
            cc_market_class: self.cc_market_class.unwrap_or_default(),
            cc_market_desc: self.cc_market_desc.unwrap_or_default(),
            cc_market_manager: self.cc_market_manager.unwrap_or_default(),
            cc_division_id: self.cc_division_id.unwrap_or(0),
            cc_division_name: self.cc_division_name.unwrap_or_default(),
            cc_company: self.cc_company.unwrap_or(0),
            cc_company_name: self.cc_company_name.unwrap_or_default(),
            cc_address: self.cc_address.unwrap_or_default(),
            cc_tax_percentage: self.cc_tax_percentage.unwrap_or_default(),
            null_bit_map: self.null_bit_map.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, Decimal};

    #[test]
    fn test_call_center_row_builder() {
        let address = Address::builder()
            .street_number(123)
            .street_name("Main St".to_string())
            .street_type("St".to_string())
            .suite_number("Suite 100".to_string())
            .city("Seattle".to_string())
            .county("King".to_string())
            .state("WA".to_string())
            .zip(98101)
            .country("United States".to_string())
            .gmt_offset(-8)
            .build();

        let tax_percentage = Decimal::new(825, 2).unwrap(); // 8.25%

        let row = CallCenterRow::builder()
            .set_cc_call_center_sk(1)
            .set_cc_call_center_id("AAAAAAAABAAAAAAA".to_string())
            .set_cc_rec_start_date_id(2450815.to_string())
            .set_cc_rec_end_date_id(2451179.to_string())
            .set_cc_closed_date_id((-1).to_string())
            .set_cc_open_date_id(2450816.to_string())
            .set_cc_name("NY Metro".to_string())
            .set_cc_class("large".to_string())
            .set_cc_employees(2)
            .set_cc_sq_ft(1138)
            .set_cc_hours("8AM-8AM".to_string())
            .set_cc_manager("Bob Belcher".to_string())
            .set_cc_market_id(6)
            .set_cc_market_class("More than other authori".to_string())
            .set_cc_market_desc("Enough employees over the".to_string())
            .set_cc_market_manager("Julius Tran".to_string())
            .set_cc_division_id(3)
            .set_cc_division_name("pri".to_string())
            .set_cc_company(6)
            .set_cc_company_name("cally".to_string())
            .set_cc_address(address)
            .set_cc_tax_percentage(tax_percentage)
            .set_null_bit_map(0)
            .build();

        // Test getters
        assert_eq!(row.get_cc_call_center_sk(), 1);
        assert_eq!(row.get_cc_call_center_id(), "AAAAAAAABAAAAAAA");
        assert_eq!(row.get_cc_name(), "NY Metro");
        assert_eq!(row.get_cc_employees(), 2);
        assert_eq!(row.get_cc_sq_ft(), 1138);
        assert_eq!(row.get_cc_tax_percentage().to_string(), "8.25");
    }

    #[test]
    fn test_call_center_row_table_row() {
        let row = CallCenterRow::builder()
            .set_cc_call_center_sk(1)
            .set_cc_call_center_id("TEST123".to_string())
            .set_cc_name("Test Center".to_string())
            .build();

        let values = row.get_values();
        assert_eq!(values.len(), 31); // 31 columns total
        assert_eq!(values[0], "1"); // cc_call_center_sk
        assert_eq!(values[1], "TEST123"); // cc_call_center_id
        assert_eq!(values[6], "Test Center"); // cc_name
    }

    #[test]
    fn test_call_center_row_clone_and_equality() {
        let row1 = CallCenterRow::builder()
            .set_cc_call_center_sk(42)
            .set_cc_name("Test Center".to_string())
            .build();

        let row2 = row1.clone();
        assert_eq!(row1, row2);
        assert_eq!(row1.get_cc_call_center_sk(), row2.get_cc_call_center_sk());
        assert_eq!(row1.get_cc_name(), row2.get_cc_name());
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods can be chained in any order
        let row = CallCenterRow::builder()
            .set_cc_employees(100)
            .set_cc_call_center_sk(5)
            .set_cc_name("Chained Builder Test".to_string())
            .set_cc_sq_ft(5000)
            .build();

        assert_eq!(row.get_cc_call_center_sk(), 5);
        assert_eq!(row.get_cc_name(), "Chained Builder Test");
        assert_eq!(row.get_cc_employees(), 100);
        assert_eq!(row.get_cc_sq_ft(), 5000);
    }
}
