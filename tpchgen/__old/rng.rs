//! Implementation of a random number generator compatible with `dbgen`.

use std::i32;

use crate::utils::{self, strings::pad_with_zeroes};

#[derive(Default, Debug, Clone, Copy)]
pub struct TpchRng {
    seed: i64,
    usage: i64,
    expected_usage_per_row: i64,
}

impl TpchRng {
    /// Constants as defined in https://github.com/electrum/tpch-dbgen/blob/master/rnd.h
    const MULTIPLIER: i64 = 16807;
    const MODULUS: i64 = 2147483647;
    const DEFAULT_SEED: i64 = 19650218;

    /// Creates a new random number generator with a given initial seed
    /// and the number of random values per row.
    pub fn new(seed: i64, uses: i64) -> Self {
        Self {
            seed,
            expected_usage_per_row: uses,
            usage: 0,
        }
    }

    /// Returns a random value uniformly picked from the range specified by
    /// `lower_bound` and `upper_bound` both inclusive.
    pub fn next_int(&mut self, lower_bound: i32, upper_bound: i32) -> i32 {
        let _ = self.next_rand();

        // This code is buggy but must be this way because we aim to have
        // bug-for-bug with the original C implementation.
        //
        // The overflow happens when high is `i32::MAX` and low is `0` and
        // the code relies on this bug.
        let range = ((upper_bound - lower_bound) as i64 + 1) as i32;
        let value_in_range =
            (((1. * self.seed as f64) / Self::MODULUS as f64) * range as f64) as i32;

        lower_bound + value_in_range
    }

    /// Updates the random number generator internal state by updating its internal
    /// seed and incrementing the usage counter.
    fn next_rand(&mut self) -> i64 {
        debug_assert!(
            self.usage <= self.expected_usage_per_row,
            "expected random to be used at most {} times per row but it was used {} times",
            self.expected_usage_per_row,
            self.usage
        );
        self.seed = (self.seed * Self::MULTIPLIER) % Self::MODULUS;
        // Increment the "use" counter for the rng.
        self.usage += 1;
        self.seed
    }

    /// Advance the seed by a specific number of rows, essentially bumps
    /// the rng state to next partition.
    fn advance_rows(&mut self, row_count: i64) {
        if self.usage != 0 {
            self.row_finished();
        }

        self.advance_seed(self.expected_usage_per_row * row_count);
    }

    /// Advances the seed to start the sequence for the next row.
    fn row_finished(&mut self) {
        self.advance_seed(self.expected_usage_per_row - self.usage);
        self.usage = 0;
    }

    /// Advance the seed after `count` calls.
    fn advance_seed(&mut self, count: i64) {
        let mut count = count;
        let mut multiplier = Self::MULTIPLIER;
        while count > 0 {
            if count % 2 != 0 {
                self.seed = (multiplier * self.seed) % Self::MODULUS;
            }
            count /= 2;
            multiplier = (multiplier * multiplier) % Self::MODULUS;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TpchLongRng {
    seed: i64,
    usage: i32,
    expected_usage_per_row: i32,
}

impl TpchLongRng {
    const MULTIPLIER: i64 = 6364136223846793005;
    const MULTIPLIER_32: i64 = 16807;
    const MODULUS_32: i64 = 2147483647;
    const INCREMENT: i64 = 1;

    /// Creates a new instance of an i64 TPC-H compatible number generator.
    pub fn new(seed: i64, expected_usage_per_row: i32) -> Self {
        Self {
            seed,
            usage: 0,
            expected_usage_per_row,
        }
    }

    /// Returns a random value between `lower_bound` and `upper_bound`.
    pub fn next_long(&mut self, lower_bound: i64, upper_bound: i64) -> i64 {
        let _ = self.next_rand();
        let value_in_range = self.seed.abs() % (upper_bound - lower_bound + 1);
        lower_bound + value_in_range
    }

    fn next_rand(&mut self) -> i64 {
        self.seed = (self.seed * Self::MULTIPLIER) + Self::INCREMENT;
        self.usage += 1;
        self.seed
    }

    fn row_finished(&mut self) {
        self.advance_seed32((self.expected_usage_per_row - self.usage) as i64);
        self.usage = 0;
    }

    fn advance_rows(&mut self, row_count: i64) {
        if self.usage != 0 {
            self.row_finished();
        }

        self.advance_seed32(self.expected_usage_per_row as i64 * row_count);
    }

    fn advance_seed32(&mut self, count: i64) {
        let mut multiplier = Self::MULTIPLIER_32;
        let mut count = count;
        while count > 0 {
            if count % 2 != 0 {
                self.seed = (multiplier * self.seed) % Self::MODULUS_32;
            }
            count /= 2;
            multiplier = (multiplier * multiplier) % Self::MODULUS_32;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TpchPhoneNumberRng {
    rng: TpchRng,
}

impl TpchPhoneNumberRng {
    /// Cardinality of the nations set.
    const NATIONS_MAX: i64 = 90;

    /// Creates a new phone number generator.
    pub fn new(seed: i64) -> Self {
        Self {
            rng: TpchRng::new(seed, 1),
        }
    }

    /// Creates a new phone number generator.
    pub fn new_with_row_usage(seed: i64, expected_row_count: i32) -> Self {
        Self {
            rng: TpchRng::new(seed, 3 * expected_row_count as i64),
        }
    }

    /// Returns the next value.
    pub fn next_value(&mut self, nation_key: i64) -> String {
        utils::strings::pad_with_zeroes(10 + (nation_key % Self::NATIONS_MAX), 2)
            + "-"
            + pad_with_zeroes(self.rng.next_int(100, 999) as i64, 3).as_str()
            + "-"
            + pad_with_zeroes(self.rng.next_int(100, 999) as i64, 3).as_str()
            + "-"
            + pad_with_zeroes(self.rng.next_int(1000, 9999) as i64, 4).as_str()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TpchAlphanumRng {
    seed: i64,
    min_len: usize,
    max_len: usize,
    rng: TpchRng,
}

impl TpchAlphanumRng {
    /// Multiplier for small length strings.
    const LOW_LENGTH_MULTIPLIER: f64 = 0.4;
    /// Multiplier for larger strings.
    const HIGH_LENGTH_MULTIPLIER: f64 = 1.6;
    /// Dictionary used for generation.
    const DICT: &'static [char] = &[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ' ', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
        'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ',',
    ];
    /// Uses per row.
    const USAGE_PER_ROW: i32 = 9;

    /// Create a new random alphanumeric string generator.
    pub fn new(seed: i64, avg_len: usize, expected_row_count: i32) -> Self {
        Self {
            seed,
            min_len: (avg_len as f64 * Self::LOW_LENGTH_MULTIPLIER) as usize,
            max_len: (avg_len as f64 * Self::HIGH_LENGTH_MULTIPLIER) as usize,
            rng: TpchRng::new(seed, (Self::USAGE_PER_ROW * expected_row_count) as i64),
        }
    }

    /// Returns the next random string.
    pub fn next_value(&mut self) -> String {
        let size = self.rng.next_int(self.min_len as i32, self.max_len as i32);
        let mut buf = vec![' '; size as usize];

        let mut index = 0;
        for i in 0..size {
            if i % 5 == 0 {
                index = self.rng.next_int(0, i32::MAX);
            }
            buf[i as usize] = Self::DICT[(index & 0x3f) as usize];
            index >>= 6;
        }

        buf.iter().cloned().collect::<String>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TpchBoundedIntRng {
    lower_bound: i32,
    upper_bound: i32,
    rng: TpchRng,
}

impl TpchBoundedIntRng {
    /// Create a new bounded random number genereator.
    pub fn new(seed: i64, lower_bound: i32, upper_bound: i32, expected_row_count: i32) -> Self {
        Self {
            lower_bound,
            upper_bound,
            rng: TpchRng::new(seed, expected_row_count as i64),
        }
    }

    /// Returns next random value between `lower_bound` and `upper_bound`.
    pub fn next_value(&mut self) -> i32 {
        self.rng.next_int(self.lower_bound, self.upper_bound)
    }
}

#[cfg(test)]
mod tests {
    use super::{TpchAlphanumRng, TpchPhoneNumberRng, TpchRng};

    #[test]
    fn can_build_a_valid_rng() {
        let mut rng = TpchRng::new(933588178, i32::MAX as i64);

        for _ in 0..1024 {
            rng.next_int(0, 1024);
        }
    }

    #[test]
    fn can_build_a_valid_phone_number_rng() {
        let mut rng = TpchPhoneNumberRng::new_with_row_usage(933588178, 32);

        for i in 0..32 as i64 {
            let _ = rng.next_value(1);
        }
    }

    #[test]
    fn can_build_a_valid_alphanum_rng() {
        let mut rng = TpchAlphanumRng::new(933588178, 20, 32);

        for _ in 0..32 {
            let s = rng.next_value();
            println!("{s}");
        }
    }
}
