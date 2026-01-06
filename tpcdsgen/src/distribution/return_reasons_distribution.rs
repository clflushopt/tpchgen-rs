use crate::distribution::FileBasedStringValuesDistribution;
use crate::error::Result;
use std::sync::OnceLock;

/// Distribution for return reasons (ReturnReasons)
pub struct ReturnReasonsDistribution;

impl ReturnReasonsDistribution {
    /// Lazy-loaded distribution instance
    fn get_distribution() -> &'static FileBasedStringValuesDistribution {
        static DISTRIBUTION: OnceLock<FileBasedStringValuesDistribution> = OnceLock::new();
        DISTRIBUTION.get_or_init(|| {
            FileBasedStringValuesDistribution::build_string_values_distribution(
                "return_reasons.dst",
                1,
                6,
            )
            .expect("Failed to load return_reasons.dst")
        })
    }

    /// Get return reason at the specified index (getValueAtIndex)
    pub fn get_return_reason_at_index(index: usize) -> Result<&'static str> {
        Self::get_distribution().get_value_at_index(0, index)
    }

    /// Get the size of the return reasons distribution
    pub fn get_size() -> usize {
        Self::get_distribution().get_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_reasons_distribution() {
        // Test that we can load the distribution
        let size = ReturnReasonsDistribution::get_size();
        assert!(
            size > 0,
            "Return reasons distribution should have at least one entry"
        );

        // Test that we can get values at valid indices
        for i in 0..size.min(5) {
            let value = ReturnReasonsDistribution::get_return_reason_at_index(i);
            assert!(value.is_ok(), "Should be able to get value at index {}", i);
            assert!(
                !value.unwrap().is_empty(),
                "Value at index {} should not be empty",
                i
            );
        }
    }

    #[test]
    fn test_return_reasons_out_of_bounds() {
        let size = ReturnReasonsDistribution::get_size();
        let result = ReturnReasonsDistribution::get_return_reason_at_index(size + 100);
        assert!(result.is_err(), "Should fail for out of bounds index");
    }
}
