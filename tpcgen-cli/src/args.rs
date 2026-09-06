//! Shared command-line argument parsing.

pub(crate) fn parse_row_group_bytes<T>(value: &str) -> Result<T, String>
where
    T: TryFrom<i128>,
    T::Error: std::fmt::Display,
{
    // A signed intermediate gives i64 and usize the same non-positive-value error.
    let bytes = value.parse::<i128>().map_err(|err| err.to_string())?;
    if bytes <= 0 {
        return Err("must be greater than zero".to_string());
    }
    T::try_from(bytes).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_group_bytes_preserves_integer_ranges() {
        assert_eq!(
            parse_row_group_bytes::<i64>(&i64::MAX.to_string()),
            Ok(i64::MAX)
        );
        assert_eq!(
            parse_row_group_bytes::<usize>(&usize::MAX.to_string()),
            Ok(usize::MAX)
        );
        assert!(parse_row_group_bytes::<i64>("9223372036854775808").is_err());
        let usize_overflow = (usize::MAX as u128 + 1).to_string();
        assert!(parse_row_group_bytes::<usize>(&usize_overflow).is_err());
    }
}
