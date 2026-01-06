use crate::row::TableRow;
use crate::types::Date;

/// Row structure for the WEB_PAGE table (WebPageRow)
#[derive(Debug, Clone)]
pub struct WebPageRow {
    null_bit_map: i64,
    wp_page_sk: i64,
    wp_page_id: String,
    wp_rec_start_date_id: i64,
    wp_rec_end_date_id: i64,
    wp_creation_date_sk: i64,
    wp_access_date_sk: i64,
    wp_autogen_flag: bool,
    wp_customer_sk: i64,
    wp_url: String,
    wp_type: String,
    wp_char_count: i32,
    wp_link_count: i32,
    wp_image_count: i32,
    wp_max_ad_count: i32,
}

impl WebPageRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        wp_page_sk: i64,
        wp_page_id: String,
        wp_rec_start_date_id: i64,
        wp_rec_end_date_id: i64,
        wp_creation_date_sk: i64,
        wp_access_date_sk: i64,
        wp_autogen_flag: bool,
        wp_customer_sk: i64,
        wp_url: String,
        wp_type: String,
        wp_char_count: i32,
        wp_link_count: i32,
        wp_image_count: i32,
        wp_max_ad_count: i32,
    ) -> Self {
        Self {
            null_bit_map,
            wp_page_sk,
            wp_page_id,
            wp_rec_start_date_id,
            wp_rec_end_date_id,
            wp_creation_date_sk,
            wp_access_date_sk,
            wp_autogen_flag,
            wp_customer_sk,
            wp_url,
            wp_type,
            wp_char_count,
            wp_link_count,
            wp_image_count,
            wp_max_ad_count,
        }
    }

    // Getters for SCD comparison (needed in WebPageRowGenerator)
    pub fn get_wp_creation_date_sk(&self) -> i64 {
        self.wp_creation_date_sk
    }

    pub fn get_wp_access_date_sk(&self) -> i64 {
        self.wp_access_date_sk
    }

    pub fn get_wp_autogen_flag(&self) -> bool {
        self.wp_autogen_flag
    }

    pub fn get_wp_customer_sk(&self) -> i64 {
        self.wp_customer_sk
    }

    pub fn get_wp_char_count(&self) -> i32 {
        self.wp_char_count
    }

    pub fn get_wp_link_count(&self) -> i32 {
        self.wp_link_count
    }

    pub fn get_wp_image_count(&self) -> i32 {
        self.wp_image_count
    }

    pub fn get_wp_max_ad_count(&self) -> i32 {
        self.wp_max_ad_count
    }

    /// Check if a column should be null based on the null bit map (shouldBeNull)
    fn should_be_null(&self, column_position: i32) -> bool {
        (self.null_bit_map & (1 << column_position)) != 0
    }

    /// Convert optional value to string or empty string if null (getStringOrNull)
    fn get_string_or_null<T: std::fmt::Display>(
        &self,
        value: Option<&T>,
        column_position: i32,
    ) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else {
            match value {
                Some(v) => v.to_string(),
                None => String::new(),
            }
        }
    }

    /// Convert key to string or empty string if null (getStringOrNullForKey)
    /// Returns empty if null OR if value is -1
    fn get_string_or_null_for_key(&self, value: i64, column_position: i32) -> String {
        if self.should_be_null(column_position) || value == -1 {
            String::new()
        } else {
            value.to_string()
        }
    }

    /// Convert boolean to Y/N string or empty string if null (getStringOrNullForBoolean)
    fn get_string_or_null_for_boolean(&self, value: bool, column_position: i32) -> String {
        if self.should_be_null(column_position) {
            String::new()
        } else if value {
            "Y".to_string()
        } else {
            "N".to_string()
        }
    }

    /// Convert julian date to date string or empty string if null (getDateStringOrNullFromJulianDays)
    /// Returns empty if null OR if value is negative
    fn get_date_string_or_null_from_julian_days(
        &self,
        julian_days: i64,
        column_position: i32,
    ) -> String {
        if self.should_be_null(column_position) || julian_days < 0 {
            String::new()
        } else {
            let date = Date::from_julian_days(julian_days as i32);
            date.to_string()
        }
    }
}

impl TableRow for WebPageRow {
    fn get_values(&self) -> Vec<String> {
        vec![
            self.get_string_or_null_for_key(self.wp_page_sk, 0),
            self.get_string_or_null(Some(&self.wp_page_id), 1),
            self.get_date_string_or_null_from_julian_days(self.wp_rec_start_date_id, 2),
            self.get_date_string_or_null_from_julian_days(self.wp_rec_end_date_id, 3),
            self.get_string_or_null_for_key(self.wp_creation_date_sk, 4),
            self.get_string_or_null_for_key(self.wp_access_date_sk, 5),
            self.get_string_or_null_for_boolean(self.wp_autogen_flag, 6),
            self.get_string_or_null_for_key(self.wp_customer_sk, 7),
            self.get_string_or_null(Some(&self.wp_url), 8),
            self.get_string_or_null(Some(&self.wp_type), 9),
            self.get_string_or_null(Some(&self.wp_char_count.to_string()), 10),
            self.get_string_or_null(Some(&self.wp_link_count.to_string()), 11),
            self.get_string_or_null(Some(&self.wp_image_count.to_string()), 12),
            self.get_string_or_null(Some(&self.wp_max_ad_count.to_string()), 13),
        ]
    }
}
