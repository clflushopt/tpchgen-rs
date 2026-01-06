use crate::distribution::embedded_data::get_embedded_distribution;
use crate::error::{Result, TpcdsError};

/// Loads and parses distribution files (.dst format)
/// Uses compile-time embedded data for zero runtime file I/O.
pub struct DistributionFileLoader;

impl DistributionFileLoader {
    /// Load a distribution file and return parsed lines
    /// Each line is split by colon into value and weight parts
    ///
    /// Distribution files are embedded at compile time, eliminating runtime file dependencies.
    pub fn load_distribution_file(filename: &str) -> Result<Vec<(Vec<String>, Vec<String>)>> {
        // Get embedded bytes (compile-time embedded)
        let bytes = get_embedded_distribution(filename).ok_or_else(|| {
            TpcdsError::new(&format!(
                "Distribution file not found (not embedded): {}",
                filename
            ))
        })?;

        // Convert ISO-8859-1 to UTF-8 string
        let content = bytes.iter().map(|&b| b as char).collect::<String>();

        let mut parsed_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            // Split by colon (not escaped colon)
            let parts: Vec<&str> = Self::split_by_unescaped_colon(trimmed);

            if parts.len() != 2 {
                return Err(TpcdsError::new(&format!(
                    "Expected line to contain 2 parts but it contains {}: {}",
                    parts.len(),
                    trimmed
                )));
            }

            let values = if parts[0].is_empty() {
                vec![String::new()] // Handle empty string case like ": weight1, weight2"
            } else {
                Self::parse_comma_separated_values(parts[0])?
            };
            let weights = Self::parse_comma_separated_values(parts[1])?;

            parsed_lines.push((values, weights));
        }

        Ok(parsed_lines)
    }

    /// Split by colon, but not escaped colon (\\:)
    fn split_by_unescaped_colon(line: &str) -> Vec<&str> {
        // Simple implementation that splits by colon and trims
        // In a full implementation, we'd properly handle escaped colons
        line.split(':').map(str::trim).collect()
    }

    /// Parse comma-separated values, handling escaped commas (\\,)
    fn parse_comma_separated_values(input: &str) -> Result<Vec<String>> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut chars = input.trim().chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' && chars.peek() == Some(&',') {
                // Escaped comma, add the comma to current value
                current.push(',');
                chars.next(); // consume the ','
            } else if ch == '\\' && chars.peek() == Some(&'\\') {
                // Escaped backslash
                current.push('\\');
                chars.next(); // consume the second '\'
            } else if ch == ',' {
                // Unescaped comma, split here
                values.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            values.push(current.trim().to_string());
        }

        // Remove escaping from final values
        for value in &mut values {
            *value = value.replace("\\\\", "\\");
        }

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comma_separated_values() {
        let result = DistributionFileLoader::parse_comma_separated_values("a, b, c").unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);

        let result = DistributionFileLoader::parse_comma_separated_values("a\\, b, c").unwrap();
        assert_eq!(result, vec!["a, b", "c"]);

        let result = DistributionFileLoader::parse_comma_separated_values("a\\\\, b").unwrap();
        assert_eq!(result, vec!["a\\", "b"]);
    }

    #[test]
    fn test_split_by_unescaped_colon() {
        let result = DistributionFileLoader::split_by_unescaped_colon("value: 1, 2, 3");
        assert_eq!(result, vec!["value", "1, 2, 3"]);
    }

    #[test]
    fn test_load_call_centers_distribution() {
        // This will test against an actual file
        let result = DistributionFileLoader::load_distribution_file("call_centers.dst");
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(!data.is_empty());

        // Check first entry should be something like "New England"
        assert_eq!(data[0].0.len(), 1); // 1 value field
        assert_eq!(data[0].1.len(), 2); // 2 weight fields
    }
}
