//! Routines to convert TPC-DS types to Arrow types

use arrow::array::builder::make_view;
use arrow::array::{
    Date32Array, Decimal128Array, Int32Array, Int64Array, PrimitiveArray, StringViewArray,
    StringViewBuilder,
};
use arrow::buffer::{BooleanBuffer, Buffer, NullBuffer, ScalarBuffer};
use arrow::datatypes::{ArrowPrimitiveType, Date32Type, Decimal128Type, Int32Type, Int64Type};
use std::fmt::{Display, Write as _};
use std::sync::Arc;
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
    nulls: NullBufferRecycler,
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
            nulls: NullBufferRecycler::default(),
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
        let mut bits = self.nulls.reclaim();
        pack_bits(&mut bits, rows, valid);

        let values = ScalarBuffer::from(values);
        let nulls = self.nulls.build(bits, rows.len());
        self.values = Some(values.clone());
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
}

/// Recycles the null bitmap allocation across batches. See [`PrimitiveArrayRecycler`].
#[derive(Debug, Default)]
struct NullBufferRecycler(Option<NullBuffer>);

impl NullBufferRecycler {
    /// Returns the previous null bitmap as an empty `Vec<u64>`, or a new one if still shared.
    fn reclaim(&mut self) -> Vec<u64> {
        let mut v = self
            .0
            .take()
            .and_then(|n| n.into_inner().into_inner().into_vec().ok())
            .unwrap_or_default();
        v.clear();
        v
    }

    /// Wrap packed bits (see [`pack_bits`]) for `len` rows, keeping a reference for reuse.
    fn build(&mut self, bits: Vec<u64>, len: usize) -> NullBuffer {
        let nulls = NullBuffer::new(BooleanBuffer::new(Buffer::from_vec(bits), 0, len));
        self.0 = Some(nulls.clone());
        nulls
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
    /// Nullable BIGINT column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the integer value for a row
    pub fn int64<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> i64,
    ) -> Int64Array {
        self.build(rows, value, |r| !is_null(nbm(r), pos))
    }

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
    /// Nullable DECIMAL(precision, scale) column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the unscaled decimal value for a row
    pub fn decimal<R>(
        &mut self,
        rows: &[R],
        precision: u8,
        scale: i8,
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> i128,
    ) -> Decimal128Array {
        self.build(rows, value, |r| !is_null(nbm(r), pos))
            .with_precision_and_scale(precision, scale)
            .unwrap()
    }

    /// Nullable DECIMAL(5,2) column
    ///
    /// Arguments: as [`Self::decimal`]
    pub fn decimal_5_2<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Decimal,
    ) -> Decimal128Array {
        self.decimal(rows, 5, 2, pos, nbm, |r| decimal_to_i128(value(r)))
    }

    /// Nullable DECIMAL(7,2) column
    ///
    /// Arguments: as [`Self::decimal`]
    pub fn decimal_7_2<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Decimal,
    ) -> Decimal128Array {
        self.decimal(rows, 7, 2, pos, nbm, |r| decimal_to_i128(value(r)))
    }

    /// Nullable DECIMAL(15,2) column
    ///
    /// Arguments: as [`Self::decimal`]
    pub fn decimal_15_2<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Decimal,
    ) -> Decimal128Array {
        self.decimal(rows, 15, 2, pos, nbm, |r| decimal_to_i128(value(r)))
    }

    /// Nullable DECIMAL(5,2) column holding whole-number GMT offsets
    ///
    /// Arguments: as [`Self::decimal`], with `value` returning the offset in hours
    pub fn gmt_offset<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> i32,
    ) -> Decimal128Array {
        self.decimal(rows, 5, 2, pos, nbm, |r| i128::from(value(r)) * 100)
    }
}

impl PrimitiveArrayRecycler<Date32Type> {
    /// Nullable DATE column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the date for a row
    pub fn date32<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Date,
    ) -> Date32Array {
        self.build(
            rows,
            |r| date_to_date32(&value(r)),
            |r| !is_null(nbm(r), pos),
        )
    }

    /// Nullable DATE column from Julian day numbers
    ///
    /// See [`julian_to_date32`]: a negative Julian day is treated as NULL.
    ///
    /// Arguments: as [`Self::date32`], with `julian` returning the Julian day for a row
    pub fn date32_julian<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        julian: impl Fn(&R) -> i64,
    ) -> Date32Array {
        self.build(
            rows,
            |r| julian(r) as i32 - UNIX_EPOCH_JULIAN,
            |r| !is_null(nbm(r), pos) && julian(r) >= 0,
        )
    }
}

/// Builds one `Utf8View` column per batch, reusing the view, data and null
/// buffer allocations from the previous batch. See [`PrimitiveArrayRecycler`].
///
/// Strings longer than 12 bytes are appended to a single data buffer per
/// batch; shorter strings are inlined in the view.
#[derive(Debug, Default)]
pub struct StringViewArrayRecycler {
    views: Option<ScalarBuffer<u128>>,
    data: Option<Buffer>,
    nulls: NullBufferRecycler,
    /// Scratch for [`Self::build_fmt`], reused across rows and batches
    scratch: String,
}

/// Views and data buffers of the batch being built by
/// [`StringViewArrayRecycler::build_with`].
#[derive(Debug)]
pub struct ViewBuilder {
    views: Vec<u128>,
    data: Vec<u8>,
}

impl ViewBuilder {
    /// Append a value. Returns `true`: the row is valid.
    #[inline]
    pub fn value(&mut self, s: &str) -> bool {
        let s = s.as_bytes();
        let offset = if s.len() > 12 {
            let offset = u32::try_from(self.data.len()).expect("string data exceeds 4GiB");
            self.data.extend_from_slice(s);
            offset
        } else {
            0
        };
        self.views.push(make_view(s, 0, offset));
        true
    }

    /// Append a NULL. Returns `false`: the row is not valid.
    #[inline]
    pub fn null(&mut self) -> bool {
        self.views.push(0);
        false
    }
}

impl StringViewArrayRecycler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an array with an entry for each row in `rows`
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `row`: appends exactly one value or null for a row and returns true
    ///   if the row is valid and false of the row was NULL..
    pub fn build_with<R>(
        &mut self,
        rows: &[R],
        mut row: impl FnMut(&R, &mut ViewBuilder) -> bool,
    ) -> StringViewArray {
        let mut b = ViewBuilder {
            views: self.reclaim_views(),
            data: self.reclaim_data(),
        };
        b.views.reserve(rows.len());
        let mut bits = self.nulls.reclaim();
        bits.reserve(rows.len().div_ceil(64));
        for chunk in rows.chunks(64) {
            let mut packed = 0u64;
            for (i, r) in chunk.iter().enumerate() {
                packed |= (row(r, &mut b) as u64) << i;
            }
            bits.push(packed.to_le());
        }
        debug_assert_eq!(b.views.len(), rows.len());

        let views = ScalarBuffer::from(b.views);
        let data = Buffer::from_vec(b.data);
        let nulls = self.nulls.build(bits, rows.len());
        self.views = Some(views.clone());
        self.data = Some(data.clone());
        let buffers: Arc<[Buffer]> = Arc::from([data]);
        if cfg!(debug_assertions) {
            StringViewArray::try_new(views, buffers, Some(nulls)).expect("valid by construction")
        } else {
            // SAFETY: every view was made by `make_view` from UTF-8 bytes that
            // are either inlined or at that offset of buffer 0, and the null
            // buffer has one bit per row.
            unsafe { StringViewArray::new_unchecked(views, buffers, Some(nulls)) }
        }
    }

    /// Build an array with an entry for each row in `rows`
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `value`: function that returns the string for a row, or `None` for NULL
    pub fn build<R>(&mut self, rows: &[R], value: impl Fn(&R) -> Option<&str>) -> StringViewArray {
        self.build_with(rows, |r, b| match value(r) {
            Some(s) => b.value(s),
            None => b.null(),
        })
    }

    /// Build an array of values formatted into a scratch `String`, without
    /// allocating per row
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `write`: writes the value for a row into &scratch and returns `true`; For NULL
    ///   writes nothing and returns `false`;
    pub fn build_fmt<R>(
        &mut self,
        rows: &[R],
        write: impl Fn(&R, &mut String) -> bool,
    ) -> StringViewArray {
        let mut scratch = std::mem::take(&mut self.scratch);
        let array = self.build_with(rows, |r, b| {
            scratch.clear();
            if write(r, &mut scratch) {
                b.value(&scratch)
            } else {
                b.null()
            }
        });
        self.scratch = scratch;
        array
    }

    /// Nullable CHAR / VARCHAR column
    ///
    /// Arguments:
    /// * `rows`: slice of rows to build the array from
    /// * `pos`: bit position in the null bitmap indicating null values
    /// * `nbm`: function to get the null bitmap for a row
    /// * `value`: function to get the string for a row
    pub fn str<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> &str,
    ) -> StringViewArray {
        self.build(rows, |r| (!is_null(nbm(r), pos)).then(|| value(r)))
    }

    /// Nullable CHAR / VARCHAR column whose value may also be absent
    ///
    /// Arguments: as [`Self::str`], with `value` returning `None` for an absent value
    pub fn opt_str<R>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> Option<&str>,
    ) -> StringViewArray {
        self.build(rows, |r| if is_null(nbm(r), pos) { None } else { value(r) })
    }

    /// Nullable CHAR / VARCHAR column holding a formatted [`Display`] value
    /// (e.g. an integer stored as CHAR)
    ///
    /// Arguments: The same as [`Self::str`], with `value` returning the value to format
    pub fn display<R, D: Display>(
        &mut self,
        rows: &[R],
        pos: u32,
        nbm: impl Fn(&R) -> i64,
        value: impl Fn(&R) -> D,
    ) -> StringViewArray {
        self.build_fmt(rows, |r, buf| {
            if is_null(nbm(r), pos) {
                return false;
            }
            write!(buf, "{}", value(r)).unwrap();
            true
        })
    }

    /// Returns the previous views buffer as an empty `Vec`, or a new one if still shared.
    fn reclaim_views(&mut self) -> Vec<u128> {
        let mut v = self
            .views
            .take()
            .and_then(|b| b.into_inner().into_vec().ok())
            .unwrap_or_default();
        v.clear();
        v
    }

    /// Returns the previous data buffer as an empty `Vec`, or a new one if still shared.
    fn reclaim_data(&mut self) -> Vec<u8> {
        let mut v = self
            .data
            .take()
            .and_then(|b| b.into_vec().ok())
            .unwrap_or_default();
        v.clear();
        v
    }
}

/// The 10 Arrow columns of an [`Address`], built by [`AddressArrayRecycler`]
#[derive(Debug)]
pub struct AddressArrays {
    pub street_number: StringViewArray,
    pub street_name: StringViewArray,
    pub street_type: StringViewArray,
    pub suite_number: StringViewArray,
    pub city: StringViewArray,
    pub county: StringViewArray,
    pub state: StringViewArray,
    pub zip: StringViewArray,
    pub country: StringViewArray,
    pub gmt_offset: Decimal128Array,
}

/// Builds the 10 [`Address`] columns per batch, recycling each column's
/// buffers. See [`PrimitiveArrayRecycler`].
#[derive(Debug, Default)]
pub struct AddressArrayRecycler {
    street_number: StringViewArrayRecycler,
    street_name: StringViewArrayRecycler,
    street_type: StringViewArrayRecycler,
    suite_number: StringViewArrayRecycler,
    city: StringViewArrayRecycler,
    county: StringViewArrayRecycler,
    state: StringViewArrayRecycler,
    zip: StringViewArrayRecycler,
    country: StringViewArrayRecycler,
    gmt_offset: PrimitiveArrayRecycler<Decimal128Type>,
}

impl AddressArrayRecycler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the address columns for each row in `rows`
    ///
    /// Each sub-field has its own null bit at `base + offset` (0 = street_number,
    /// 1 = street_name, ..., 9 = gmt_offset), matching the per-column
    /// null_bit_map layout.
    ///
    /// Arguments:
    /// * `rows`: rows to build the arrays from
    /// * `base`: bit position of street_number in the null bitmap
    /// * `nbm`: function to get the null bitmap for a row
    /// * `address`: function to get the address for a row
    pub fn build<R>(
        &mut self,
        rows: &[R],
        base: u32,
        nbm: impl Fn(&R) -> i64,
        address: impl Fn(&R) -> &Address,
    ) -> AddressArrays {
        let nbm = &nbm;
        let address = &address;
        AddressArrays {
            street_number: self
                .street_number
                .display(rows, base, nbm, |r| address(r).get_street_number()),
            street_name: self.street_name.build_fmt(rows, |r, buf| {
                if is_null(nbm(r), base + 1) {
                    return false;
                }
                // as Address::get_street_name, without allocating
                let a = address(r);
                write!(buf, "{} {}", a.get_street_name1(), a.get_street_name2()).unwrap();
                true
            }),
            street_type: self
                .street_type
                .str(rows, base + 2, nbm, |r| address(r).get_street_type()),
            suite_number: self
                .suite_number
                .str(rows, base + 3, nbm, |r| address(r).get_suite_number()),
            city: self
                .city
                .str(rows, base + 4, nbm, |r| address(r).get_city()),
            county: self
                .county
                .opt_str(rows, base + 5, nbm, |r| address(r).get_county()),
            state: self
                .state
                .str(rows, base + 6, nbm, |r| address(r).get_state()),
            zip: self.zip.build_fmt(rows, |r, buf| {
                if is_null(nbm(r), base + 7) {
                    return false;
                }
                write!(buf, "{:05}", address(r).get_zip()).unwrap();
                true
            }),
            country: self
                .country
                .str(rows, base + 8, nbm, |r| address(r).get_country()),
            gmt_offset: self
                .gmt_offset
                .gmt_offset(rows, base + 9, nbm, |r| address(r).get_gmt_offset()),
        }
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
    fn test_recycler_int64_keeps_negative_values() {
        let rows: Vec<i64> = vec![-5, 0, 7];
        let mut rec = PrimitiveArrayRecycler::<Int64Type>::new();
        let a = rec.int64(&rows, 0, |r| (*r == 0) as i64, |r| *r);
        assert_eq!(a.value(0), -5);
        assert!(a.is_null(1));
        assert_eq!(a.value(2), 7);
    }

    #[test]
    fn test_recycler_decimal_precisions() {
        let rows: Vec<i64> = vec![150, 250];
        let mut rec = PrimitiveArrayRecycler::<Decimal128Type>::new();
        let d = |r: &i64| Decimal::new(*r, 2);
        let a = rec.decimal_5_2(&rows, 0, |_| 0, d);
        assert_eq!((a.precision(), a.scale()), (5, 2));
        let b = rec.decimal_15_2(&rows, 0, |_| 0, d);
        assert_eq!((b.precision(), b.scale()), (15, 2));
        assert_eq!(b.value(1), 250);
        let c = rec.gmt_offset(&rows, 0, |r| (*r == 250) as i64, |r| (*r / 100) as i32);
        assert_eq!((c.precision(), c.scale()), (5, 2));
        assert_eq!(c.value(0), 100);
        assert!(c.is_null(1));
    }

    #[test]
    fn test_recycler_date32() {
        let rows: Vec<i64> = vec![UNIX_EPOCH_JULIAN as i64, -1, UNIX_EPOCH_JULIAN as i64 + 1];
        let mut rec = PrimitiveArrayRecycler::<Date32Type>::new();
        let a = rec.date32_julian(&rows, 0, |r| (*r > UNIX_EPOCH_JULIAN as i64) as i64, |r| *r);
        assert_eq!(a.value(0), 0);
        assert!(a.is_null(1)); // negative julian
        assert!(a.is_null(2)); // bitmap bit set
        let b = rec.date32(&rows[..1], 0, |_| 0, |_| Date::new(1970, 1, 1));
        assert_eq!(b.value(0), 0);
    }

    #[test]
    fn test_string_recycler_reuses_buffers_after_drop() {
        // mix of inlined (<= 12 bytes) and out-of-line strings
        let rows: Vec<String> = (0..70).map(|i| "x".repeat(i % 20)).collect();
        let mut rec = StringViewArrayRecycler::new();
        let a = rec.build(&rows, |r| (r.len() != 7).then_some(r.as_str()));
        assert_eq!(a.len(), 70);
        assert_eq!(a.null_count(), 4); // lengths cycle every 20 rows
        for (i, r) in rows.iter().enumerate() {
            if r.len() == 7 {
                assert!(a.is_null(i));
            } else {
                assert_eq!(a.value(i), r.as_str(), "row {i}");
            }
        }
        assert_eq!(a.data_buffers().len(), 1);
        let views_ptr = a.views().as_ptr();
        let data_ptr = a.data_buffers()[0].as_ptr();
        let null_ptr = a.nulls().unwrap().validity().as_ptr();
        drop(a);

        // Sole reference dropped: the same allocations come back
        let b = rec.build(&rows[..20], |r| Some(r.as_str()));
        assert_eq!(b.views().as_ptr(), views_ptr);
        assert_eq!(b.data_buffers()[0].as_ptr(), data_ptr);
        assert_eq!(b.nulls().unwrap().validity().as_ptr(), null_ptr);
        assert_eq!(b.null_count(), 0);
        assert_eq!(b.value(7), "xxxxxxx");
        assert_eq!(b.value(19), "x".repeat(19));

        // Still held by the caller: fresh allocations are used
        let c = rec.build(&rows[..2], |_| None);
        assert_ne!(c.views().as_ptr(), views_ptr);
        assert_eq!(c.null_count(), 2);
        assert_eq!(b.value(19), "x".repeat(19));
    }

    #[test]
    fn test_string_recycler_bitmap_and_option() {
        let rows: Vec<(i64, Option<&str>)> = vec![
            (0, Some("a")),
            (1, Some("b")),
            (0, None),
            (0, Some("a long string here")),
        ];
        let mut rec = StringViewArrayRecycler::new();
        let a = rec.str(&rows, 0, |r| r.0, |r| r.1.unwrap_or("-"));
        assert_eq!(a.value(0), "a");
        assert!(a.is_null(1));
        assert_eq!(a.value(2), "-");
        assert_eq!(a.value(3), "a long string here");
        let b = rec.opt_str(&rows, 0, |r| r.0, |r| r.1);
        assert_eq!(b.value(0), "a");
        assert!(b.is_null(1));
        assert!(b.is_null(2));
        assert_eq!(b.value(3), "a long string here");
    }

    #[test]
    fn test_string_recycler_display_and_fmt() {
        let rows: Vec<(i64, i64)> = vec![(0, 7), (1, 8), (0, 1234567890123)];
        let mut rec = StringViewArrayRecycler::new();
        let a = rec.display(&rows, 0, |r| r.0, |r| r.1);
        assert_eq!(a.value(0), "7");
        assert!(a.is_null(1));
        assert_eq!(a.value(2), "1234567890123"); // > 12 bytes: out of line
        let b = rec.build_fmt(&rows, |r, buf| {
            write!(buf, "{:05}", r.1).unwrap();
            r.1 != 8
        });
        assert_eq!(b.value(0), "00007");
        assert!(b.is_null(1));
        assert_eq!(b.value(2), "1234567890123");
    }

    #[test]
    fn test_integer_sk_opt_handles_absent_keys() {
        assert_eq!(integer_sk_opt(0, 0, -1), None);
        assert_eq!(integer_sk_opt(1, 0, i64::from(i32::MAX) + 1), None);
        assert_eq!(integer_sk_opt(0, 0, i64::from(i32::MAX)), Some(i32::MAX));
    }
}
