use crate::{check_argument, check_state, error::Result, TpcdsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: i32,
    month: i32,
    day: i32,
}

impl Date {
    // Constants matching Java implementation
    pub const JULIAN_DATA_START_DATE: i64 = 2450815; // toJulianDays(Date::new(1998, 1, 1))
    pub const JULIAN_DATA_END_DATE: i64 = 2453005; // toJulianDays(Date::new(2003, 12, 31))
    pub const TODAYS_DATE: Date = Date {
        year: 2003,
        month: 1,
        day: 8,
    };
    pub const JULIAN_TODAYS_DATE: i32 = 2452648; // toJulianDays(TODAYS_DATE) = 2003-01-08
    pub const CURRENT_QUARTER: i32 = 1;
    pub const CURRENT_WEEK: i32 = 2;

    pub const DATE_MAXIMUM: Date = Date {
        year: 2002,
        month: 12,
        day: 31,
    };
    pub const DATE_MINIMUM: Date = Date {
        year: 1998,
        month: 1,
        day: 1,
    };
    pub const JULIAN_DATE_MAXIMUM: i32 = 2452640; // toJulianDays(DATE_MAXIMUM)
    pub const JULIAN_DATE_MINIMUM: i32 = 2450815; // toJulianDays(DATE_MINIMUM)

    pub const WEEKDAY_NAMES: [&'static str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];

    // Month day cumulative arrays (0-indexed for convenience, but month 0 is unused)
    const MONTH_DAYS: [i32; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    const MONTH_DAYS_LEAP_YEAR: [i32; 13] =
        [0, 0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

    // Const version for compile-time date creation (no validation)
    pub const fn new(year: i32, month: i32, day: i32) -> Self {
        Date { year, month, day }
    }

    // Safe version with validation
    pub fn new_validated(year: i32, month: i32, day: i32) -> Result<Self> {
        check_argument!(year > 0, "Year must be a positive value");
        check_argument!(
            month > 0 && month <= 12,
            "Month must be a number between 1 and 12 (inclusive)"
        );
        check_argument!(day > 0 && day <= Self::get_days_in_month(month, year)?,
                       "Day must be a positive value and cannot exceed the maximum number of days in the month");

        Ok(Date { year, month, day })
    }

    // Algorithm: Fleigel and Van Flandern (CACM, vol 11, #10, Oct. 1968, p. 657)
    pub fn from_julian_days(julian_days: i32) -> Self {
        let l = julian_days + 68569;
        let n = (4 * l) / 146097;
        let l = l - (146097 * n + 3) / 4;
        let i = (4000 * (l + 1)) / 1461001;
        let l = l - (1461 * i) / 4 + 31;
        let j = (80 * l) / 2447;

        let day = l - (2447 * j) / 80;
        let l = j / 11;
        let month = j + 2 - 12 * l;
        let year = 100 * (n - 49) + i + l;

        Self::new(year, month, day)
    }

    // http://quasar.as.utexas.edu/BillInfo/JulianDatesG.html
    pub fn to_julian_days(&self) -> i32 {
        let mut month = self.month;
        let mut year = self.year;

        // Start years in March so you don't have to account for February.
        if month <= 2 {
            month += 12;
            year -= 1;
        }

        let days_bce_in_julian_epoch = 1721118; // Days Before the Common Era (before year 1) in the Julian Epoch

        // The month calculation looks convoluted, but can be thought of as follows:
        // There are a little over 30.6 (153/5) days in a month (excluding February)
        // Subtract 3 months because we start from the third month and don't include the current month
        // (153/5 * 3 = 459/5)
        // adding another 2/5 gets you 31 days at the right times.
        self.day +
            (153 * month - 457) / 5 +
            365 * year + year / 4 - year / 100 + year / 400 + // 365 days in a year + leap years
            days_bce_in_julian_epoch + 1
    }

    // This is NOT a correct computation of leap years.
    // There is a bug in the C code that doesn't handle century years correctly.
    pub fn is_leap_year(year: i32) -> bool {
        year % 4 == 0
    }

    pub fn get_days_in_year(year: i32) -> i32 {
        if Self::is_leap_year(year) {
            366
        } else {
            365
        }
    }

    pub fn get_day(&self) -> i32 {
        self.day
    }

    pub fn get_year(&self) -> i32 {
        self.year
    }

    pub fn get_month(&self) -> i32 {
        self.month
    }

    fn get_days_in_month(month: i32, year: i32) -> Result<i32> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 => {
                if Self::is_leap_year(year) {
                    Ok(29)
                } else {
                    Ok(28)
                }
            }
            _ => {
                check_state!(false, &format!("Invalid value for month {}", month));
                unreachable!()
            }
        }
    }

    // The ordinal reference into the calendar distribution for a given date
    pub fn get_day_index(&self) -> i32 {
        Self::get_days_through_first_of_month(self) + self.get_day()
    }

    fn get_days_through_first_of_month(date: &Date) -> i32 {
        if Self::is_leap_year(date.get_year()) {
            Self::MONTH_DAYS_LEAP_YEAR[date.get_month() as usize]
        } else {
            Self::MONTH_DAYS[date.get_month() as usize]
        }
    }

    pub fn compute_first_date_of_month(&self) -> Result<Self> {
        Ok(Self::new(self.year, self.month, 1))
    }

    pub fn compute_last_date_of_month(&self) -> Result<Self> {
        // Copies a bug in the C code that adds all the days in the year
        // through the first of month instead of just the number of days in the month
        let julian_days =
            self.to_julian_days() - self.day + Self::get_days_through_first_of_month(self);
        Ok(Self::from_julian_days(julian_days))
    }

    pub fn compute_same_day_last_year(&self) -> Result<Self> {
        let mut day = self.day;
        if Self::is_leap_year(self.year) && self.month == 2 && self.day == 29 {
            day = 28;
        }
        Ok(Self::new(self.year - 1, self.month, day))
    }

    pub fn compute_same_day_last_quarter(&self) -> Result<Self> {
        let quarter = (self.month - 1) / 3; // zero-indexed quarter number
        let julian_start_of_quarter = Self::new(self.year, quarter * 3 + 1, 1).to_julian_days();
        let julian_date = self.to_julian_days();
        let distance_from_start = julian_date - julian_start_of_quarter;

        let last_quarter = if quarter > 0 { quarter - 1 } else { 3 };
        let last_quarter_year = if quarter > 0 {
            self.year
        } else {
            self.year - 1
        };
        let julian_start_of_previous_quarter =
            Self::new(last_quarter_year, last_quarter * 3 + 1, 1).to_julian_days();

        Ok(Self::from_julian_days(
            julian_start_of_previous_quarter + distance_from_start,
        ))
    }

    // Uses the doomsday algorithm to calculate the day of the week.
    // The doomsday algorithm is based on the knowledge that our calendar
    // repeats itself every 400 years. Additionally, there are certain easy
    // to remember dates in the year that always fall on the same day as each
    // other. The day of the week on which these dates fall is referred to as doomsday.
    // https://en.wikipedia.org/wiki/Doomsday_rule
    pub fn compute_day_of_week(&self) -> i32 {
        // doomsdays for the first year of each century in a 400 year cycle
        let century_anchors = [3, 2, 0, 5];

        // Dates in each month that are known to fall on the same day of the week as each other.
        // The zero at index zero is just a place holder because months are 1-indexed.
        // Other values of zero refer to the last day of the previous month.
        let mut known = [0, 3, 0, 0, 4, 9, 6, 11, 8, 5, 10, 7, 12];

        let year = self.get_year();
        if Self::is_leap_year(self.get_year()) {
            // adjust the known dates for January and February
            known[1] = 4;
            known[2] = 1;
        }

        // calculate the doomsday for the century
        let mut century_index = year / 100;
        century_index -= 15; // the year 1500 would be at index zero
        century_index %= 4; // which century are we in in the 400 year cycle
        let century_anchor = century_anchors[century_index as usize];

        // and then calculate the doomsday for the year
        let year_of_century = year % 100;
        let q = year_of_century / 12;
        let r = year_of_century % 12;
        let s = r / 4;
        let mut doomsday = century_anchor + q + r + s;
        doomsday %= 7;

        // finally, calculate the day of week for our date
        let mut result = self.get_day();
        result -= known[self.get_month() as usize];
        while result < 0 {
            result += 7;
        }
        while result > 6 {
            result -= 7;
        }

        result += doomsday;
        result % 7
    }

    // Pre-computed constants for the static methods
    pub fn julian_data_start_date() -> i64 {
        Self::JULIAN_DATA_START_DATE
    }

    pub fn julian_data_end_date() -> i64 {
        Self::JULIAN_DATA_END_DATE
    }

    pub fn todays_date() -> Date {
        Self::TODAYS_DATE
    }

    pub fn julian_todays_date() -> i32 {
        Self::JULIAN_TODAYS_DATE
    }

    pub fn date_maximum() -> Date {
        Self::DATE_MAXIMUM
    }

    pub fn date_minimum() -> Date {
        Self::DATE_MINIMUM
    }

    pub fn julian_date_maximum() -> i32 {
        Self::JULIAN_DATE_MAXIMUM
    }

    pub fn julian_date_minimum() -> i32 {
        Self::JULIAN_DATE_MINIMUM
    }

    // Helper function to convert Julian date to formatted string
    pub fn julian_to_date_string(julian_days: i64) -> String {
        let date = Self::from_julian_days(julian_days as i32);
        date.to_string()
    }

    // Convenience methods for cleaner API
    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> i32 {
        self.month
    }

    pub fn day(&self) -> i32 {
        self.day
    }

    pub fn day_of_week(&self) -> i32 {
        self.compute_day_of_week()
    }

    pub fn day_of_year(&self) -> i32 {
        self.get_day_index()
    }

    pub fn last_day_of_month(&self) -> Date {
        // Using unwrap is safe here because we're constructing from valid dates
        self.compute_last_date_of_month().unwrap()
    }

    pub fn same_day_last_year(&self) -> Date {
        self.compute_same_day_last_year().unwrap()
    }

    pub fn same_day_last_quarter(&self) -> Date {
        self.compute_same_day_last_quarter().unwrap()
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_creation() {
        let date = Date::new(2003, 1, 8);
        assert_eq!(date.get_year(), 2003);
        assert_eq!(date.get_month(), 1);
        assert_eq!(date.get_day(), 8);
    }

    #[test]
    fn test_julian_conversion() {
        let date = Date::new(1998, 1, 1);
        let julian = date.to_julian_days();
        assert_eq!(julian as i64, Date::JULIAN_DATA_START_DATE);

        let back = Date::from_julian_days(julian);
        assert_eq!(back, date);
    }

    #[test]
    fn test_leap_year_bug() {
        // Test the intentional bug - 1900 should not be a leap year but this function says it is
        assert!(Date::is_leap_year(1900)); // Bug: should be false
        assert!(Date::is_leap_year(2000)); // Correct: should be true
        assert!(!Date::is_leap_year(2001)); // Correct: should be false
    }

    #[test]
    fn test_display() {
        let date = Date::new(2003, 1, 8);
        assert_eq!(format!("{}", date), "2003-01-08");
    }
}
