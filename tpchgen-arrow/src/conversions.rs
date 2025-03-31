//! Routines to convert TPCH types to Arrow types

use arrow::array::{ByteView, GenericByteViewArray, StringViewArray};
use arrow::buffer::{Buffer, ScalarBuffer};
use tpchgen::dates::TPCHDate;
use tpchgen::decimal::TPCHDecimal;
use tpchgen::generators::LineItemStatus;

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
/// Example
/// ```
/// # use arrow::array::StringViewArray;
/// # use arrow::array::Array;
/// # use tpchgen_arrow::conversions::string_view_array_from_display_iter;
/// let values = vec![
///   "Hello",
///   "This is a string that is longer than 12 bytes and will be stored in a buffer",
///   "World",
/// ];
/// // This will convert the string values to a StringViewArray with minimal overhead
/// let actual = string_view_array_from_display_iter(values.clone());
/// assert_eq!(actual.len(), 3);
/// // check the values in the StringViewArray built with the builder
/// let expected = StringViewArray::from_iter_values(values);
/// assert_eq!(actual, expected);
/// ```
pub fn string_view_array_from_display_iter<I>(values: I) -> StringViewArray
where
    I: IntoIterator<Item: std::fmt::Display>,
{
    // Construct a `StringViewArray` directly from parts rather than StringViewBuilder
    // to avoid an extra copy of the data.
    let values = values.into_iter();

    // format all values directly into the buffer
    let mut buffer: Vec<u8> = Vec::with_capacity (31*1024*1024);
    let mut views = Vec::with_capacity(values.size_hint().0);


    for v in values {
        use std::io::Write;
        let start_offset = buffer.len();
        write!(&mut buffer, "{v}").unwrap();
        let length = buffer.len() - start_offset; // length of the string we just wrote

        // implementation adapted from Arrow's GenericByteViewBuilder
        // https://docs.rs/arrow-array/54.3.1/src/arrow_array/builder/generic_bytes_view_builder.rs.html#286

        let len_u32 = length as u32; // length of the string
        if length <= 12 { // inline view
            let s = &buffer[start_offset..start_offset + length]; // slice of the string in the buffer
            let mut view_buffer = [0; 16];
            view_buffer[0..4].copy_from_slice(&len_u32.to_le_bytes());
            view_buffer[4..4 + length].copy_from_slice(s);
            views.push(u128::from_le_bytes(view_buffer));
            // data in buffer is not needed, so clear space for next string
            buffer.truncate(start_offset);
        } else {
            let prefix =  u32::from_le_bytes(buffer[start_offset..start_offset+4].try_into().unwrap());
            let view = ByteView {
                length: len_u32, // length of the string
                prefix,
                buffer_index: 0, // we only make a single buffer
                offset: start_offset as u32,
            };
            views.push(u128::from(view));
        }
    }

    let views = ScalarBuffer::from(views); // convert to ScalarBuffer<u128>
    // all values are in the single, buffer
    let buffers = vec![Buffer::from(buffer)];
    let nulls = None; // no nulls in data

    // SAFETY: valid by construction
    unsafe { GenericByteViewArray::new_unchecked(views, buffers, nulls) }

}

/// Coverts an iterator of [`ReturnFlag`] to an Arrow [`StringViewArray`] avoiding
/// an extra copy of the data
///
/// Example
/// ```
/// # use arrow::array::StringViewArray;
/// # use arrow::array::Array;
/// # use tpchgen_arrow::conversions::string_view_array_from_return_flag_iter;
/// let return_flags = vec![
///    "A", // 'A' for Accepted
///    "R", // 'R' for Returned
///    "N", // 'N' for No"
/// ];
/// // This will convert the ReturnItem values to a StringViewArray with minimal overhead
/// let actual = string_view_array_from_return_flag_iter(return_flags);
/// assert_eq!(actual.len(), 3);
/// // check the values in the StringViewArray build with the builder
/// let expected = StringViewArray::from_iter_values(["A", "R", "N"]);
/// assert_eq!(actual, expected);
/// ```
pub fn string_view_array_from_return_flag_iter<I>(values: I) -> StringViewArray
where
    I: IntoIterator<Item = &'static str>,
{
    let values = values.into_iter();
    // All valid values of LineItemStatus are less than 12 bytes long and thus
    // is entirely inlined, we can use precomputed u128 views
    let views: ScalarBuffer<u128> = values.map(|s| {
        // Views in Arrow's StringViewArray are represented as u128 values
        // low 32 bits of the u128 represent the length (1)
        // then the inlined the string 'A' (0x41), 'R' (0x52), or 'N' (0x4E)
        // https://docs.rs/arrow/latest/arrow/array/struct.GenericByteViewArray.html#layout-views-and-buffers
        match s {
            "A" => 0x00000000000000000000004100000001,
            "R" => 0x00000000000000000000005200000001,
            "N" => 0x00000000000000000000004E00000001,
            _ => panic!("Invalid return flag value: {s}. This function only supports the values 'A', 'R', and 'N'."),
        }
    }).collect();
    let buffers = vec![]; // all values are inlined in the u128, so no need for a buffer
    let nulls = None;

    // SAFETY: valid by construction
    unsafe { GenericByteViewArray::new_unchecked(views, buffers, nulls) }
}

/// Converts an iterator of [`LineItemStatus`] to an Arrow [`StringViewArray`] avoiding
/// an extra copy of the data
///
/// Example
/// ```
/// # use arrow::array::StringViewArray;
/// # use arrow::array::Array;
/// # use tpchgen::generators::LineItemStatus;
/// # use tpchgen_arrow::conversions::string_view_array_from_line_item_status_iter;
/// let line_item_statuses = vec![
///    LineItemStatus::Fulfilled,
///    LineItemStatus::Open,
/// ];
/// // This will convert the LineItemStatus values to a StringViewArray with minimal overhead
/// let actual = string_view_array_from_line_item_status_iter(line_item_statuses);
/// assert_eq!(actual.len(), 2);
/// // check the values in the StringViewArray build with the builder
/// let expected = StringViewArray::from_iter_values(["F", "O"]);
/// assert_eq!(actual, expected);
/// ```
pub fn string_view_array_from_line_item_status_iter<I>(values: I) -> StringViewArray
where
    I: IntoIterator<Item = LineItemStatus>,
{
    let values = values.into_iter();
    // All valid values of LineItemStatus are less than 12 bytes long and thus
    // is entirely inlined, we can use precomputed u128 views
    let views: ScalarBuffer<u128> = values
        .map(|status| {
            // There are only two valid values for LineItemStatus: 'F' and 'P'
            // Views in Arrow's StringViewArray are represented as u128 values
            // low 32 bits of the u128 represent the length (1)
            // then the inlined the string 'F' (0x46) or 'P' (0x4F)
            // https://docs.rs/arrow/latest/arrow/array/struct.GenericByteViewArray.html#layout-views-and-buffers
            match status {
                LineItemStatus::Fulfilled => 0x00000000000000000000004600000001,
                LineItemStatus::Open => 0x00000000000000000000004F00000001,
            }
        })
        .collect();
    let buffers = vec![]; // all values are inlined in the u128, so no need for a buffer
    let nulls = None;

    // SAFETY: valid by construction
    unsafe { GenericByteViewArray::new_unchecked(views, buffers, nulls) }
}

/// Converts an iterator of shipmode fields to an Arrow [`StringViewArray`] avoiding
/// an extra copy of the data
///
/// This takes advantage of the fact there are only 7 distinct values for
/// l_shipmode in the TPCH schema and all are less than 12 bytes so can be inlined
///
/// All valid values for `l_shipmode` in the TPCH schema are less than 12 bytes
/// long
///
/// ```sql
/// DataFusion CLI v46.0.1
/// +------------+
/// | l_shipmode |
/// +------------+
/// | TRUCK      |
/// | RAIL       |
/// | REG AIR    |
/// | FOB        |
/// | MAIL       |
/// | AIR        |
/// | SHIP       |
/// +------------+
/// ```
///
/// Example
/// ```
/// # use arrow::array::StringViewArray;
/// # use arrow::array::Array;
/// # use tpchgen_arrow::conversions::string_view_array_from_shipmode_iter;
/// let ship_modes = vec![
///    "TRUCK",
///    "RAIL",
///    "REG AIR",
///    "FOB",
///    "MAIL",
///    "AIR",
///    "SHIP",
/// ];
/// // This will convert the LineItemStatus values to a StringViewArray with minimal overhead
/// let actual = string_view_array_from_shipmode_iter(ship_modes.clone());
/// assert_eq!(actual.len(), 7);
/// // check the values in the StringViewArray build with the builder
/// let expected = StringViewArray::from_iter_values(ship_modes);
/// assert_eq!(actual, expected);
/// ```
pub fn string_view_array_from_shipmode_iter<I>(values: I) -> StringViewArray
where
    I: IntoIterator<Item = &'static str>,
{
    let values = values.into_iter();
    // we know the only valid values for shipmode in the TPCH schema are less than
    // 12 bytes long and can be inlined in a u128, so we can use precomputed views
    let views: ScalarBuffer<u128> = values.map(|ship_mode| {
        match ship_mode {
            // Views in Arrow's StringViewArray are represented as u128 values
            // low 32 bits of the u128 represent the length (1)
            // https://docs.rs/arrow/latest/arrow/array/struct.GenericByteViewArray.html#layout-views-and-buffers
            "TRUCK" => 0x000000000000004b4355525400000005, // 0x4b43555254 = 'TRUCK'
            "RAIL" => 0x000000000000004c49415200000004,    // 0x4c494152 = 'RAIL'
            "REG AIR" => 0x000000000000005249412047455200000007, // 0x5249472041474552 = 'REG AIR'
            "FOB" => 0x00000000000000424f4600000003,       // 0x424f46 = 'FOB'
            "MAIL" => 0x000000000000004c49414d00000004,    // 0x4c49414d = 'MAIL'
            "AIR" => 0x0000000000000052494100000003,       // 0x524941 = 'AIR'
            "SHIP" => 0x000000000000005049485300000004,    // 0x50495048 = 'SHIP'
            _ => panic!(
                "Invalid ship mode value: {ship_mode}. This function only supports the 7 valid values for l_shipmode in the TPCH schema."
            )
        }
    }).collect();
    let buffers = vec![]; // all values are inlined in the u128, so no need for a buffer
    let nulls = None;

    // SAFETY: valid by construction
    unsafe { GenericByteViewArray::new_unchecked(views, buffers, nulls) }
}

/// Converts an iterator of shipmode fields to an Arrow [`StringViewArray`] avoiding
/// an extra copy of the data
///
/// This takes advantage of the fact there are only 4 distinct values for
/// l_shipinstruct in the TPCH schema
///
/// ```sql
/// DataFusion CLI v46.0.1
/// +-------------------+
/// | l_shipinstruct    |
/// +-------------------+
/// | COLLECT COD       |
/// | TAKE BACK RETURN  |
/// | DELIVER IN PERSON |
/// | NONE              |
/// +-------------------+
/// ```
///
/// Example
/// ```
/// # use arrow::array::StringViewArray;
/// # use arrow::array::Array;
/// # use tpchgen_arrow::conversions::string_view_array_from_shipinstruct_iter;
/// let ship_instructs = vec![
///    "COLLECT COD",
///    "TAKE BACK RETURN",
///    "DELIVER IN PERSON",
///    "NONE",
/// ];
/// // This will convert the instructions values to a StringViewArray with minimal overhead
/// let actual = string_view_array_from_shipinstruct_iter(ship_instructs.clone());
/// assert_eq!(actual.len(), 4);
/// // check the values in the StringViewArray build with the builder
/// let expected = StringViewArray::from_iter_values(ship_instructs);
/// assert_eq!(actual, expected);
/// ```
pub fn string_view_array_from_shipinstruct_iter<I>(values: I) -> StringViewArray
where
    I: IntoIterator<Item = &'static str>,
{
    let values = values.into_iter();
    // values less than 12 bytes are inlined in the u128
    // values larger than 12 bytes will be stored in a buffer, but since we know
    // the valid values for `l_shipinstruct` we precompute it here
    let mut buffer: Vec<u8> = vec![];
    // "COLLECT COD" is 11 bytes long, so it can be inlined in a u128
    // 0x444f43205443454c4c4f43 = 'COLLECT COD'
    let collect_cod_view = 0x00444f43205443454c4c4f430000000b;
    // "TAKE BACK RETURN" is 16 bytes long, so it must stored in the buffer
    // 0x454b4154 = 'TAKE' (prefix)
    buffer.extend_from_slice(b"TAKE BACK RETURN");
    let take_back_return_view = 0x0000000000000000454b415400000010;
    // "DELIVER IN PERSON" is 17 bytes long, so it must stored in the buffer
    // 0x494c4544 = 'DELI' (prefix)
    buffer.extend_from_slice(b"DELIVER IN PERSON");
    let deliver_in_person_view = 0x0000001000000000494c454400000011;
    // "NONE" is 4 bytes long, so it can be inlined in a u128
    let none_view = 0x0000000000000000454e4f4e00000004; // 0x454e4f4e = 'NONE'

    let views: ScalarBuffer<u128> = values.map(|s| {
        match s {
            "COLLECT COD" => collect_cod_view,
            "TAKE BACK RETURN" => take_back_return_view,
            "DELIVER IN PERSON" => deliver_in_person_view,
            "NONE" => none_view,
            _ => panic!("Invalid ship instruction value: {s}. This function only supports the 4 valid values for l_shipinstruct in the TPCH schema.")
        }
    }).collect();
    let buffers = vec![Buffer::from(buffer)];
    let nulls = None;

    // SAFETY: valid by construction
    unsafe { GenericByteViewArray::new_unchecked(views, buffers, nulls) }
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
