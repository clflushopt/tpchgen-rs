use chrono::NaiveDate;
use std::{
    fmt::{Display, Formatter},
    sync::LazyLock,
};

/// The value of 1970-01-01 in the date generator system
pub const GENERATED_DATE_EPOCH_OFFSET: i32 = 83966;
pub const MIN_GENERATE_DATE: i32 = 92001;
const CURRENT_DATE: i32 = 95168;
pub const TOTAL_DATE_RANGE: i32 = 2557;

/// Month boundaries for a standard (non-leap) year
const MONTH_YEAR_DAY_START: [i32; 13] =
    [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

static DATE_INDEX: LazyLock<Vec<NaiveDate>> = LazyLock::new(make_date_index);

pub struct GenerateUtils;

impl GenerateUtils {
    /// Calculates row count for a specific part of the data
    pub fn calculate_row_count(
        scale_base: i32,
        scale_factor: f64,
        part: i32,
        part_count: i32,
    ) -> i64 {
        let total_row_count = (scale_base as f64 * scale_factor) as i64;
        let mut row_count = total_row_count / part_count as i64;

        if part == part_count {
            // for the last part, add the remainder rows
            row_count += total_row_count % part_count as i64;
        }

        row_count
    }

    /// Calculates start index for a specific part of the data
    pub fn calculate_start_index(
        scale_base: i32,
        scale_factor: f64,
        part: i32,
        part_count: i32,
    ) -> i64 {
        let total_row_count = (scale_base as f64 * scale_factor) as i64;
        let rows_per_part = total_row_count / part_count as i64;
        rows_per_part * (part as i64 - 1)
    }
}

/// Represents a date (day/year)
///
/// Example display: 1992-01-01
#[derive(Debug, Clone, PartialEq)]
pub struct TPCHDate {
    inner: NaiveDate,
}

impl Display for TPCHDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl TPCHDate {
    /// Converts a generated date to an epoch date
    pub fn new(generated_date: i32) -> Self {
        Self {
            inner: Self::format_date(generated_date - GENERATED_DATE_EPOCH_OFFSET),
        }
    }

    /// Formats a date from epoch date format
    fn format_date(epoch_date: i32) -> NaiveDate {
        let idx = epoch_date - (MIN_GENERATE_DATE - GENERATED_DATE_EPOCH_OFFSET);
        DATE_INDEX[idx as usize]
    }

    /// Format money value (convert to decimal)
    pub fn format_money(value: i64) -> String {
        format!("{:.2}", value as f64 / 100.0)
    }

    /// Checks if a date is in the past
    pub const fn is_in_past(date: i32) -> bool {
        Self::julian(date) <= CURRENT_DATE
    }

    /// Converts to julian date format
    const fn julian(date: i32) -> i32 {
        let mut offset = date - MIN_GENERATE_DATE;
        let mut result = MIN_GENERATE_DATE;

        loop {
            let year = result / 1000;
            let year_end = year * 1000 + 365 + if Self::is_leap_year(year) { 1 } else { 0 };

            if result + offset <= year_end {
                break;
            }

            offset -= year_end - result + 1;
            result += 1000;
        }

        result + offset
    }

    /// Check if a year is a leap year
    const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && year % 100 != 0
    }
}

/// Creates the date index used for lookups
fn make_date_index() -> Vec<NaiveDate> {
    let mut dates = Vec::with_capacity(TOTAL_DATE_RANGE as usize);

    for i in 0..TOTAL_DATE_RANGE {
        dates.push(make_date(i + 1));
    }

    dates
}

/// Create a chrono date from an index
const fn make_date(index: i32) -> NaiveDate {
    let y = julian(index + MIN_GENERATE_DATE - 1) / 1000;
    let d = julian(index + MIN_GENERATE_DATE - 1) % 1000;

    let mut m = 0;
    while d > MONTH_YEAR_DAY_START[m as usize] + leap_year_adjustment(y, m) {
        m += 1;
    }

    let dy =
        d - MONTH_YEAR_DAY_START[(m - 1) as usize] - if is_leap_year(y) && m > 2 { 1 } else { 0 };

    // Create date from year, month, day
    NaiveDate::from_ymd_opt(1900 + y, m as u32, dy as u32).unwrap()
}

/// Helpers duplicated to avoid circular references
const fn julian(date: i32) -> i32 {
    let mut offset = date - MIN_GENERATE_DATE;
    let mut result = MIN_GENERATE_DATE;

    loop {
        let year = result / 1000;
        let year_end = year * 1000 + 365 + if is_leap_year(year) { 1 } else { 0 };

        if result + offset <= year_end {
            break;
        }

        offset -= year_end - result + 1;
        result += 1000;
    }

    result + offset
}

const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && year % 100 != 0
}

const fn leap_year_adjustment(year: i32, month: i32) -> i32 {
    if is_leap_year(year) && month >= 2 {
        1
    } else {
        0
    }
}
