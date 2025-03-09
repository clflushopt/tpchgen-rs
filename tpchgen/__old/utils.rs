pub mod strings {
    const COLUMN_SEPARATOR: &str = "|";

    pub fn pad_with_zeroes(value: i64, length: usize) -> String {
        pad_start(value.to_string().as_str(), length, '0')
    }

    fn pad_start(value: &str, min_length: usize, pad_char: char) -> String {
        if value.len() >= min_length {
            return value.to_string();
        }

        let mut orig = "".to_string();
        for _ in value.len()..min_length {
            orig.push(pad_char);
        }

        orig.push_str(value);
        orig.to_string()
    }
}
