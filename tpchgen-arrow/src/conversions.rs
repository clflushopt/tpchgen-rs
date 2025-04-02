//! Routines to convert TPCH types to Arrow types

use arrow::array::{StringViewArray, StringViewBuilder};
use std::fmt::Write;
use tpchgen::dates::TPCHDate;
use tpchgen::decimal::TPCHDecimal;

/// Convert a TPCHDecimal to an Arrow Decimal(15,2)
#[inline(always)]
pub fn to_arrow_decimal(value: TPCHDecimal) -> i128 {
    // TPCH decimals are stored as i64 with 2 decimal places, so
    // we can simply convert to i128 directly
    value.into_inner() as i128
}

/// Convert a TPCH date to an Arrow Date32.
///
/// * Arrow `Date32` are days since the epoch (1970-01-01)
/// * [`TPCHDate`]s are days since MIN_GENERATE_DATE (1992-01-01)
///
/// ```
/// use chrono::NaiveDate;
/// use tpchgen::dates::TPCHDate;
/// let arrow_epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
///  let tpch_epoch = NaiveDate::from_ymd_opt(1992, 1, 1).unwrap();
/// // the difference between the two epochs is 8035 days
/// let day_offset = (tpch_epoch - arrow_epoch).num_days();
/// let day_offset: i32 = day_offset.try_into().unwrap();
///  assert_eq!(day_offset, TPCHDate::UNIX_EPOCH_OFFSET);
/// ```
#[inline(always)]
pub fn to_arrow_date32(value: TPCHDate) -> i32 {
    value.to_unix_epoch()
}

/// Converts an iterator of TPCH decimals to an Arrow Decimal128Array
pub fn decimal128_array_from_iter<I>(values: I) -> arrow::array::Decimal128Array
where
    I: Iterator<Item = TPCHDecimal>,
{
    let values = values.map(to_arrow_decimal);
    arrow::array::Decimal128Array::from_iter_values(values)
        .with_precision_and_scale(15, 2)
        // safe to unwrap because 15,2 is within the valid range for Decimal128 (38)
        .unwrap()
}

/// Coverts an iterator of displayable values to an Arrow StringViewArray
///
/// This results in an extra copy of the data, which could be avoided for some types
pub fn string_view_array_from_display_iter<I>(values: I) -> StringViewArray
where
    I: Iterator<Item: std::fmt::Display>,
{
    let mut buffer = String::new();
    let values = values.into_iter();
    let size_hint = values.size_hint().0;
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        buffer.clear();
        write!(&mut buffer, "{v}").unwrap();
        builder.append_value(&buffer);
    }
    builder.finish()
}

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
