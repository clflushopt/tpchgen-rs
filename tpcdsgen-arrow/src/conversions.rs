//! Routines to convert TPC-DS types to Arrow types

use arrow::array::{
    Decimal128Array, Int32Array, Int64Array, PrimitiveArray, StringViewArray, StringViewBuilder,
};
use arrow::buffer::{BooleanBuffer, Buffer, NullBuffer, ScalarBuffer};
use arrow::datatypes::{ArrowPrimitiveType, Decimal128Type, Int32Type, Int64Type};
use tpcdsgen::types::{Address, Date, Decimal};

/// Julian day number for the Unix epoch (1970-01-01)
const UNIX_EPOCH_JULIAN: i32 = 2440588;

/// Convert a TPC-DS Decimal to an i128 value suitable for Decimal128Array.
///
/// TPC-DS Decimal stores the number as an unscaled integer (e.g. 1234 means 12.34
/// when precision=2). Arrow Decimal128 also stores unscaled integers, so we just
/// cast the i64 to i128.
#[inline(always)]
pub fn decimal_to_i128(d: Decimal) -> i128 {
    d.get_number() as i128
}

/// Convert a TPC-DS Date to an Arrow Date32 (days since Unix epoch 1970-01-01).
///
/// TPC-DS Date is stored as a Julian day number internally. Julian day 2440588
/// corresponds to 1970-01-01.
#[inline(always)]
pub fn date_to_date32(d: &Date) -> i32 {
    d.to_julian_days() - UNIX_EPOCH_JULIAN
}

/// Convert a Julian day i64 to an Arrow Date32. Returns None if julian_days < 0.
#[inline(always)]
pub fn julian_to_date32(julian_days: i64) -> Option<i32> {
    if julian_days < 0 {
        None
    } else {
        Some(julian_days as i32 - UNIX_EPOCH_JULIAN)
    }
}

/// Build a TPC-DS DECIMAL(5,2) array from unscaled integer values.
pub fn decimal128_5_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(5, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(7,2) array from unscaled integer values.
pub fn decimal128_7_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(7, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(15,2) array from unscaled integer values.
pub fn decimal128_15_2_array(values: impl IntoIterator<Item = Option<i128>>) -> Decimal128Array {
    Decimal128Array::from_iter(values)
        .with_precision_and_scale(15, 2)
        .unwrap()
}

/// Build a TPC-DS DECIMAL(5,2) array from whole-number GMT offsets.
pub fn gmt_offset_decimal128_array(
    values: impl IntoIterator<Item = Option<i32>>,
) -> Decimal128Array {
    Decimal128Array::from_iter(
        values
            .into_iter()
            .map(|value| value.map(|value| i128::from(value) * 100)),
    )
    .with_precision_and_scale(5, 2)
    .unwrap()
}

/// Build a StringViewArray from an iterator of &str values (non-nullable).
pub fn string_view_array_from_iter<'a, I>(values: I) -> StringViewArray
where
    I: Iterator<Item = &'a str>,
{
    let values: Vec<&str> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        builder.append_value(v);
    }
    builder.finish()
}

/// Build a StringViewArray from an iterator of optional &str values (nullable).
pub fn string_view_array_from_opt_iter<'a, I>(values: I) -> StringViewArray
where
    I: Iterator<Item = Option<&'a str>>,
{
    let values: Vec<Option<&str>> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        match v {
            Some(s) => builder.append_value(s),
            None => builder.append_null(),
        }
    }
    builder.finish()
}

/// Build a StringViewArray from an iterator of owned String values (nullable).
pub fn string_view_array_from_string_opt_iter<I>(values: I) -> StringViewArray
where
    I: Iterator<Item = Option<String>>,
{
    let values: Vec<Option<String>> = values.collect();
    let size_hint = values.len();
    let mut builder = StringViewBuilder::with_capacity(size_hint);
    for v in values {
        match v {
            Some(ref s) => builder.append_value(s.as_str()),
            None => builder.append_null(),
        }
    }
    builder.finish()
}

/// Convert a boolean value to "Y" or "N" static str.
#[inline(always)]
pub fn bool_to_yn(b: bool) -> &'static str {
    if b {
        "Y"
    } else {
        "N"
    }
}

/// Check whether the bit at `pos` is set in a null bitmap, indicating a NULL value.
#[inline(always)]
pub fn is_null(nbm: i64, pos: u32) -> bool {
    (nbm >> pos) & 1 != 0
}

/// Return `Some(val)` unless the null bitmap bit at `pos` is set.
#[inline(always)]
pub fn opt<T>(nbm: i64, pos: u32, val: T) -> Option<T> {
    if is_null(nbm, pos) {
        None
    } else {
        Some(val)
    }
}

/// Return a checked Arrow Int32 value unless the null bitmap bit is set.
#[inline(always)]
pub fn integer_opt(nbm: i64, pos: u32, value: i64) -> Option<i32> {
    if is_null(nbm, pos) {
        None
    } else {
        Some(i32::try_from(value).expect("TPC-DS INTEGER value exceeds i32 range"))
    }
}

/// Return `Some(sk)` unless null bitmap bit is set OR sk < 0 (sentinel for absent FK).
#[inline(always)]
pub fn sk_opt(nbm: i64, pos: u32, sk: i64) -> Option<i64> {
    if is_null(nbm, pos) || sk < 0 {
        None
    } else {
        Some(sk)
    }
}

/// Return a checked Arrow Int32 surrogate key unless null or an absent-key sentinel.
#[inline(always)]
pub fn integer_sk_opt(nbm: i64, pos: u32, sk: i64) -> Option<i32> {
    if is_null(nbm, pos) || sk < 0 {
        None
    } else {
        Some(i32::try_from(sk).expect("TPC-DS INTEGER surrogate key exceeds i32 range"))
    }
}

/// Builds one primitive column per batch, column-at-a-time, reusing the value
/// and null buffer allocations from the previous batch.
///
/// Keeps a reference to the value and null buffers of the last array built. On
/// the next build, each buffer whose only remaining reference is ours is
/// converted back into a `Vec` and refilled; otherwise a fresh `Vec` is used.
#[derive(Debug)]
pub struct PrimitiveArrayRecycler<T: ArrowPrimitiveType> {
    values: Option<ScalarBuffer<T::Native>>,
    nulls: Option<NullBuffer>,
}

impl<T: ArrowPrimitiveType> Default for PrimitiveArrayRecycler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ArrowPrimitiveType> PrimitiveArrayRecycler<T> {
    pub fn new() -> Self {
        Self {
            values: None,
            nulls: None,
        }
    }

    /// Build an array with an entry for each row in `rows`
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `value`: function that returns the value for a row
    /// * `valid`: function that returns true if the row is non-null
    pub fn build<R>(
        &mut self,
        rows: &[R],
        value: impl Fn(&R) -> T::Native,
        valid: impl Fn(&R) -> bool,
    ) -> PrimitiveArray<T> {
        let mut values = self.reclaim_values();
        values.reserve(rows.len());
        values.extend(rows.iter().map(value));
        let mut bits = self.reclaim_bits();
        pack_bits(&mut bits, rows, valid);

        let values = ScalarBuffer::from(values);
        let nulls = NullBuffer::new(BooleanBuffer::new(Buffer::from_vec(bits), 0, rows.len()));
        self.values = Some(values.clone());
        self.nulls = Some(nulls.clone());
        PrimitiveArray::new(values, Some(nulls))
    }

    /// Returns the previous value buffer as an empty `Vec`, or a new one if still shared.
    fn reclaim_values(&mut self) -> Vec<T::Native> {
        let mut v = self
            .values
            .take()
            .and_then(|b| b.into_inner().into_vec().ok())
            .unwrap_or_default();
        v.clear();
        v
    }

    /// Returns the previous null bitmap as an empty `Vec<u64>`, or a new one if still shared.
    fn reclaim_bits(&mut self) -> Vec<u64> {
        let mut v = self
            .nulls
            .take()
            .and_then(|n| n.into_inner().into_inner().into_vec().ok())
            .unwrap_or_default();
        v.clear();
        v
    }
}

/// Pack `valid(row)` for each row into `dst` as little-endian 64-bit words,
/// lowest bit first (Arrow validity bitmap layout). Trailing bits are zero.
fn pack_bits<R>(dst: &mut Vec<u64>, rows: &[R], valid: impl Fn(&R) -> bool) {
    dst.reserve(rows.len().div_ceil(64));
    for chunk in rows.chunks(64) {
        let mut packed = 0u64;
        for (i, r) in chunk.iter().enumerate() {
            packed |= (valid(r) as u64) << i;
        }
        dst.push(packed.to_le());
    }
}

impl PrimitiveArrayRecycler<Int32Type> {
    /// Nullable INTEGER column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the integer value for a row
    pub fn int32<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> i32,
    ) -> Int32Array {
        self.build(rows, value, |r| !is_null(nbm(r), pos))
    }

    /// Nullable INTEGER surrogate key
    ///
    /// See [`integer_sk_opt`] for details on how nulls and absent keys are handled.
    ///
    /// Arguments: as [`Self::int32`], with `sk` returning the surrogate key for a row
    pub fn int32_sk<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        sk: impl Fn(&R) -> i64,
    ) -> Int32Array {
        self.build(
            rows,
            |r| sk(r) as i32,
            |r| !is_null(nbm(r), pos) && sk(r) >= 0,
        )
    }
}

impl PrimitiveArrayRecycler<Int64Type> {
    /// Nullable BIGINT surrogate key
    ///
    /// See [`sk_opt`] for details on how nulls and absent keys are handled.
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `sk`: function to get the surrogate key for a row
    pub fn int64_sk<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        sk: impl Fn(&R) -> i64,
    ) -> Int64Array {
        self.build(rows, &sk, |r| !is_null(nbm(r), pos) && sk(r) >= 0)
    }
}

impl PrimitiveArrayRecycler<Decimal128Type> {
    /// Nullable DECIMAL(7,2) column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the decimal value for a row
    pub fn decimal_7_2<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Decimal,
    ) -> Decimal128Array {
        self.build(
            rows,
            |r| decimal_to_i128(value(r)),
            |r| !is_null(nbm(r), pos),
        )
        .with_precision_and_scale(7, 2)
        .unwrap()
    }
}

/// Expand an [`Address`] into 10 individual column arrays (street_number, street_name,
/// street_type, suite_number, city, county, state, zip, country, gmt_offset).
///
/// Returns the 10 arrays as a flat tuple in the order listed above.
pub fn address_columns<'a>(
    rows: impl Iterator<Item = (&'a Address, i64, u32)> + 'a,
) -> (
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    StringViewArray,
    Decimal128Array,
) {
    let rows: Vec<_> = rows.collect();
    let street_number =
        string_view_array_from_string_opt_iter(rows.iter().map(|(a, nbm, base)| {
            if is_null(*nbm, *base) {
                None
            } else {
                Some(a.get_street_number().to_string())
            }
        }));
    let mut street_name_b = StringViewBuilder::with_capacity(rows.len());
    let mut street_type_b = StringViewBuilder::with_capacity(rows.len());
    let mut suite_number_b = StringViewBuilder::with_capacity(rows.len());
    let mut city_b = StringViewBuilder::with_capacity(rows.len());
    let mut county_b = StringViewBuilder::with_capacity(rows.len());
    let mut state_b = StringViewBuilder::with_capacity(rows.len());
    let mut zip_b = StringViewBuilder::with_capacity(rows.len());
    let mut country_b = StringViewBuilder::with_capacity(rows.len());

    // Each address sub-field has its own null bit at base+offset (0=street_number,
    // 1=street_name, ..., 9=gmt_offset), matching the per-column null_bit_map layout.
    for (a, nbm, base) in &rows {
        if is_null(*nbm, *base + 1) {
            street_name_b.append_null();
        } else {
            street_name_b.append_value(a.get_street_name());
        }
        if is_null(*nbm, *base + 2) {
            street_type_b.append_null();
        } else {
            street_type_b.append_value(a.get_street_type());
        }
        if is_null(*nbm, *base + 3) {
            suite_number_b.append_null();
        } else {
            suite_number_b.append_value(a.get_suite_number());
        }
        if is_null(*nbm, *base + 4) {
            city_b.append_null();
        } else {
            city_b.append_value(a.get_city());
        }
        match a.get_county() {
            Some(c) if !is_null(*nbm, *base + 5) => county_b.append_value(c),
            _ => county_b.append_null(),
        }
        if is_null(*nbm, *base + 6) {
            state_b.append_null();
        } else {
            state_b.append_value(a.get_state());
        }
        if is_null(*nbm, *base + 7) {
            zip_b.append_null();
        } else {
            zip_b.append_value(format!("{:05}", a.get_zip()));
        }
        if is_null(*nbm, *base + 8) {
            country_b.append_null();
        } else {
            country_b.append_value(a.get_country());
        }
    }
    let gmt_offset = gmt_offset_decimal128_array(rows.iter().map(|(a, nbm, base)| {
        if is_null(*nbm, *base + 9) {
            None
        } else {
            Some(a.get_gmt_offset())
        }
    }));
    (
        street_number,
        street_name_b.finish(),
        street_type_b.finish(),
        suite_number_b.finish(),
        city_b.finish(),
        county_b.finish(),
        state_b.finish(),
        zip_b.finish(),
        country_b.finish(),
        gmt_offset,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn test_decimal_to_i128() {
        let d = Decimal::new(12345, 2);
        assert_eq!(decimal_to_i128(d), 12345);
    }

    #[test]
    fn test_whole_number_decimal128_array_scales_values() {
        let array = gmt_offset_decimal128_array([Some(-5), None, Some(9)]);

        assert_eq!(array.value(0), -500);
        assert!(array.is_null(1));
        assert_eq!(array.value(2), 900);
        assert_eq!(array.precision(), 5);
        assert_eq!(array.scale(), 2);
    }

    #[test]
    fn test_julian_to_date32_epoch() {
        // Julian day 2440588 = 1970-01-01, so offset should be 0
        assert_eq!(julian_to_date32(2440588), Some(0));
    }

    #[test]
    fn test_julian_to_date32_negative() {
        assert_eq!(julian_to_date32(-1), None);
    }

    #[test]
    fn test_bool_to_yn() {
        assert_eq!(bool_to_yn(true), "Y");
        assert_eq!(bool_to_yn(false), "N");
    }

    #[test]
    fn test_integer_opt_boundaries() {
        assert_eq!(integer_opt(0, 0, i64::from(i32::MIN)), Some(i32::MIN));
        assert_eq!(integer_opt(0, 0, i64::from(i32::MAX)), Some(i32::MAX));
        assert_eq!(integer_opt(1, 0, i64::from(i32::MAX) + 1), None);
    }

    #[test]
    #[should_panic(expected = "TPC-DS INTEGER value exceeds i32 range")]
    fn test_integer_opt_rejects_overflow() {
        integer_opt(0, 0, i64::from(i32::MAX) + 1);
    }

    #[test]
    fn test_recycler_reuses_buffers_after_drop() {
        let rows: Vec<i64> = (0..100).collect();
        let mut rec = PrimitiveArrayRecycler::<Int32Type>::new();
        let a = rec.int32_sk(
            &rows,
            0,
            |r| (r % 7 == 0) as i64,
            |r| if *r == 50 { -1 } else { *r },
        );
        assert_eq!(a.len(), 100);
        assert!(a.is_null(0)); // bitmap bit set
        assert!(a.is_null(50)); // negative sk
        assert_eq!(a.value(1), 1);
        assert_eq!(a.null_count(), 16); // 15 multiples of 7 + row 50
        let ptr = a.values().as_ptr();
        let null_ptr = a.nulls().unwrap().validity().as_ptr();
        drop(a);

        // Sole reference dropped: the same allocations come back
        let b = rec.int32_sk(&rows[..10], 0, |_| 0, |r| *r);
        assert_eq!(b.values().as_ptr(), ptr);
        assert_eq!(b.nulls().unwrap().validity().as_ptr(), null_ptr);
        assert_eq!(b.null_count(), 0);
        assert_eq!(b.values(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Still held by the caller: a fresh allocation is used, values are still right
        let c = rec.int32_sk(&rows[..3], 0, |_| 0, |_| -1);
        assert_ne!(c.values().as_ptr(), ptr);
        assert_ne!(c.nulls().unwrap().validity().as_ptr(), null_ptr);
        assert_eq!(c.null_count(), 3);
        assert_eq!(b.values(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_recycler_decimal_bitmap_over_64_rows() {
        let rows: Vec<i64> = (0..130).collect();
        let mut rec = PrimitiveArrayRecycler::<Decimal128Type>::new();
        let a = rec.decimal_7_2(
            &rows,
            3,
            |r| if *r == 64 { 1 << 3 } else { 0 },
            |r| Decimal::new(*r * 100, 2),
        );
        assert_eq!(a.precision(), 7);
        assert_eq!(a.scale(), 2);
        assert_eq!(a.null_count(), 1);
        assert!(a.is_null(64));
        assert_eq!(a.value(129), 12900);
    }

    #[test]
    fn test_integer_sk_opt_handles_absent_keys() {
        assert_eq!(integer_sk_opt(0, 0, -1), None);
        assert_eq!(integer_sk_opt(1, 0, i64::from(i32::MAX) + 1), None);
        assert_eq!(integer_sk_opt(0, 0, i64::from(i32::MAX)), Some(i32::MAX));
    }
}
