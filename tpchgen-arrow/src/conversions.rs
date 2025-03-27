//! Routines to convert TPCH types to Arrow types

use tpchgen::dates::TPCHDate;
use tpchgen::decimal::TPCHDecimal;

/// Convert a TPCHDecimal to an Arrow Decimal(15,2)
#[inline(always)]
pub fn to_arrow_decimal(value: TPCHDecimal) -> i128 {
    // TPCH decimals are stored as i64 with 2 decimal places, so
    // we can simply convert to i128 directly
    value.into_inner() as i128
}

/// Convert a TPCH date to an Arrow Date32
#[inline(always)]
pub fn to_arrow_date32(value: TPCHDate) -> i32 {
    value.into_inner() + TPCHDATE_TO_DATE32_OFFSET
}

/// Number of days that must be added to a TPCH date to get an Arrow `Date32` value.
///
/// * Arrow `Date32` are days since the epoch (1970-01-01)
/// * [`TPCHDate`]s are days since MIN_GENERATE_DATE (1992-01-01)
///
/// This value is `8035` because `1992-01-01` is `8035` days after `1970-01-01`
/// ```
/// use chrono::NaiveDate;
/// use tpchgen_arrow::conversions::TPCHDATE_TO_DATE32_OFFSET;
/// let arrow_epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
///  let tpch_epoch = NaiveDate::from_ymd_opt(1992, 1, 1).unwrap();
/// // the difference between the two epochs is 8035 days
/// let day_offset = (tpch_epoch - arrow_epoch).num_days();
/// let day_offset: i32 = day_offset.try_into().unwrap();
///  assert_eq!(day_offset, TPCHDATE_TO_DATE32_OFFSET);
/// ```
pub const TPCHDATE_TO_DATE32_OFFSET: i32 = 8035;

// test to ensure that the conversion functions are correct
#[cfg(test)]
mod tests {
    use super::*;
    use tpchgen::dates::MIN_GENERATE_DATE;

    #[test]
    fn test_to_arrow_decimal() {
        let value = TPCHDecimal::new(123456789);
        assert_eq!(to_arrow_decimal(value), 123456789);
    }

    #[test]
    fn test_to_arrow_date32() {
        let value = TPCHDate::new(MIN_GENERATE_DATE);
        assert_eq!(to_arrow_date32(value), 8035);

        let value = TPCHDate::new(MIN_GENERATE_DATE + 100);
        assert_eq!(to_arrow_date32(value), 8135);

        let value = TPCHDate::new(MIN_GENERATE_DATE + 1234);
        assert_eq!(to_arrow_date32(value), 9269);
    }
}
