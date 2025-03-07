use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;
use crate::distribution::*;


/// Core implementation of TPC-H's random number generator
pub struct AbstractRandomInt {
    // Constants from TPC-H spec
    multiplier: i64,
    modulus: i64,

    seed: i64,
    expected_usage_per_row: usize,
    usage: usize,
}

impl AbstractRandomInt {
    pub fn new(seed: i64, expected_usage_per_row: usize) -> Self {
        Self {
            multiplier: 16807,
            modulus: 2147483647, // 2^31 - 1
            seed,
            expected_usage_per_row,
            usage: 0,
        }
    }

    /// Generate a random integer in range [low_value, high_value]
    pub fn next_int(&mut self, low_value: i32, high_value: i32) -> i32 {
        self.next_rand();

        // This code is strange because we must maintain the bugs in the
        // original TPC-H generator code.

        // This will result in overflow when high is max int and low is 0,
        // which is a bug since you will get a value outside of the
        // specified range. There is code that relies on this bug.
        let int_range = (high_value - low_value + 1) as i64;
        let double_range = int_range as f64;
        let value_in_range = ((1.0 * self.seed as f64 / self.modulus as f64) * double_range) as i32;

        low_value + value_in_range
    }

    /// Generate next random number
    fn next_rand(&mut self) -> i64 {
        if self.usage >= self.expected_usage_per_row {
            panic!(
                "Expected random to be used only {} times per row",
                self.expected_usage_per_row
            );
        }

        self.seed = (self.seed * self.multiplier) % self.modulus;
        self.usage += 1;
        self.seed
    }

    /// Mark current row as finished and advance to next row
    pub fn row_finished(&mut self) {
        self.advance_seed((self.expected_usage_per_row - self.usage) as i64);
        self.usage = 0;
    }

    /// Advance multiple rows
    pub fn advance_rows(&mut self, row_count: usize) {
        // Finish current row if needed
        if self.usage != 0 {
            self.row_finished();
        }

        // Advance seed for all rows
        self.advance_seed((self.expected_usage_per_row as i64) * (row_count as i64));
    }

    /// Advance seed by specified count efficiently
    fn advance_seed(&mut self, mut count: i64) {
        let mut multiplier = self.multiplier;

        while count > 0 {
            if count % 2 != 0 {
                self.seed = (multiplier * self.seed) % self.modulus;
            }

            // Integer division, truncates
            count = count / 2;
            multiplier = (multiplier * multiplier) % self.modulus;
        }
    }
}

/// Random integer generator
pub struct RandomInt {
    inner: AbstractRandomInt,
}

impl RandomInt {
    pub fn new(seed: i64, expected_usage_per_row: usize) -> Self {
        Self {
            inner: AbstractRandomInt::new(seed, expected_usage_per_row),
        }
    }

    pub fn next_int(&mut self, low_value: i32, high_value: i32) -> i32 {
        self.inner.next_int(low_value, high_value)
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }
}

/// Random 64-bit integer generator for large scale factors
pub struct RandomLong {
    // Constants from TPC-H spec for 64-bit generator
    multiplier: i64,
    increment: i64,

    seed: i64,
    expected_usage_per_row: usize,
    usage: usize,
}

impl RandomLong {
    pub fn new(seed: i64, expected_usage_per_row: usize) -> Self {
        Self {
            multiplier: 6364136223846793005,
            increment: 1,
            seed,
            expected_usage_per_row,
            usage: 0,
        }
    }

    pub fn next_long(&mut self, low_value: i64, high_value: i64) -> i64 {
        self.next_rand();

        let value_in_range = (self.seed.abs()) % (high_value - low_value + 1);

        low_value + value_in_range
    }

    fn next_rand(&mut self) -> i64 {
        if self.usage >= self.expected_usage_per_row {
            panic!(
                "Expected random to be used only {} times per row",
                self.expected_usage_per_row
            );
        }

        self.seed = self
            .seed
            .wrapping_mul(self.multiplier)
            .wrapping_add(self.increment);
        self.usage += 1;
        self.seed
    }

    pub fn row_finished(&mut self) {
        // For the 64-bit case, TPC-H actually uses the 32-bit advance method
        self.advance_seed_32((self.expected_usage_per_row - self.usage) as i64);
        self.usage = 0;
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        // Finish current row if needed
        if self.usage != 0 {
            self.row_finished();
        }

        // Advance the seed
        self.advance_seed_32((self.expected_usage_per_row as i64) * (row_count as i64));
    }

    // TPC-H uses this 32-bit method even for 64-bit numbers
    fn advance_seed_32(&mut self, mut count: i64) {
        let mut multiplier_32: i64 = 16807;
        let modulus_32: i64 = 2147483647;

        while count > 0 {
            if count % 2 != 0 {
                self.seed = (multiplier_32 * self.seed) % modulus_32;
            }

            // Integer division, truncates
            count = count / 2;
            let new_multiplier = (multiplier_32 * multiplier_32) % modulus_32;
            multiplier_32 = new_multiplier;
        }
    }
}

/// A random integer generator bounded to a specific range
pub struct RandomBoundedInt {
    inner: RandomInt,
    low_value: i32,
    high_value: i32,
}

impl RandomBoundedInt {
    pub fn new(seed: i64, low_value: i32, high_value: i32) -> Self {
        Self::new_with_expected_row_count(seed, low_value, high_value, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        low_value: i32,
        high_value: i32,
        expected_row_count: usize,
    ) -> Self {
        Self {
            inner: RandomInt::new(seed, expected_row_count),
            low_value,
            high_value,
        }
    }

    pub fn next_value(&mut self) -> i32 {
        self.inner.next_int(self.low_value, self.high_value)
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }
}

/// A random 64-bit integer generator bounded to a specific range
pub struct RandomBoundedLong {
    random_long: Option<RandomLong>,
    random_int: Option<RandomInt>,
    low_value: i64,
    high_value: i64,
}

impl RandomBoundedLong {
    pub fn new(seed: i64, use_64bits: bool, low_value: i64, high_value: i64) -> Self {
        Self::new_with_expected_row_count(seed, use_64bits, low_value, high_value, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        use_64bits: bool,
        low_value: i64,
        high_value: i64,
        expected_row_count: usize,
    ) -> Self {
        let random_long = if use_64bits {
            Some(RandomLong::new(seed, expected_row_count))
        } else {
            None
        };

        let random_int = if !use_64bits {
            Some(RandomInt::new(seed, expected_row_count))
        } else {
            None
        };

        Self {
            random_long,
            random_int,
            low_value,
            high_value,
        }
    }

    pub fn next_value(&mut self) -> i64 {
        if let Some(ref mut random_long) = self.random_long {
            random_long.next_long(self.low_value, self.high_value)
        } else if let Some(ref mut random_int) = self.random_int {
            random_int.next_int(self.low_value as i32, self.high_value as i32) as i64
        } else {
            panic!("Neither RandomLong nor RandomInt is initialized");
        }
    }

    pub fn row_finished(&mut self) {
        if let Some(ref mut random_long) = self.random_long {
            random_long.row_finished();
        } else if let Some(ref mut random_int) = self.random_int {
            random_int.row_finished();
        }
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        if let Some(ref mut random_long) = self.random_long {
            random_long.advance_rows(row_count);
        } else if let Some(ref mut random_int) = self.random_int {
            random_int.advance_rows(row_count);
        }
    }
}

/// TextPool stores a large pool of pre-generated text that follows the TPC-H grammar
pub struct TextPool {
    text_pool: Vec<u8>,
    text_pool_size: usize,
}
// These need to be added to store parsed distributions
struct ParsedDistributions {
    grammars: ParsedDistribution,
    noun_phrases: ParsedDistribution,
    verb_phrases: ParsedDistribution,
}

impl TextPool {
    const MAX_SENTENCE_LENGTH: usize = 256;
    /// Create a new TextPool from a pre-generated text byte array
    pub fn new(text_pool: Vec<u8>, text_pool_size: usize) -> Self {
        Self {
            text_pool,
            text_pool_size,
        }
    }

    /// Create a default sized TextPool (300MB) using the given distributions
    pub fn create_default_text_pool(distributions: &Distributions) -> Self {
        const DEFAULT_TEXT_POOL_SIZE: usize = 300 * 1024 * 1024;
        Self::generate(DEFAULT_TEXT_POOL_SIZE, distributions, &|_| {})
    }

    /// Generate a TextPool of the specified size
    pub fn generate(
        size: usize,
        distributions: &Distributions,
        progress_monitor: &dyn Fn(f64),
    ) -> Self {
        let start = std::time::Instant::now();
        println!("Creating new TextPool...");
        // Parse the distributions first
        let parsed_distributions = ParsedDistributions {
            grammars: ParsedDistribution::new(distributions.get_grammars()),
            noun_phrases: ParsedDistribution::new(distributions.get_noun_phrase()),
            verb_phrases: ParsedDistribution::new(distributions.get_verb_phrase()),
        };

        let mut output = ByteArrayBuilder::new(size + Self::MAX_SENTENCE_LENGTH);
        let mut random_int = RandomInt::new(933588178, i32::MAX as usize);

        while output.length() < size {
            Self::generate_sentence(
                &parsed_distributions,
                distributions,
                &mut output,
                &mut random_int,
            );
            progress_monitor(output.length() as f64 / size as f64);
        }

        // Trim to exact size
        if output.length() > size {
            output.erase(output.length() - size);
        }

        println!("Created new TextPool in {} ms", start.elapsed().as_millis());

        Self {
            text_pool: output.get_bytes(),
            text_pool_size: output.length(),
        }
    }

    /// Get the size of the text pool
    pub fn size(&self) -> usize {
        self.text_pool_size
    }

    /// Get a substring from the text pool
    pub fn get_text(&self, begin: usize, end: usize) -> String {
        if end > self.text_pool_size {
            panic!(
                "Index {} is beyond end of text pool (size = {})",
                end, self.text_pool_size
            );
        }

        // This is safe because the pool is ASCII text
        String::from_utf8_lossy(&self.text_pool[begin..end]).to_string()
    }

    /// Generate a sentence according to TPC-H grammar rules
    fn generate_sentence(
        parsed: &ParsedDistributions,
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RandomInt,
    ) {
        let syntax = distributions.get_grammars().random_value(random);
        let index = parsed.grammars.get_random_index(random);
        let tokens = parsed.grammars.get_tokens(index);

        let max_length = syntax.len();
        for i in (0..max_length).step_by(2) {
            match syntax.chars().nth(i).unwrap() {
                'V' => Self::generate_verb_phrase(parsed, distributions, builder, random),
                'N' => Self::generate_noun_phrase(parsed, distributions, builder, random),

                'P' => {
                    let preposition = distributions.get_prepositions().random_value(random);
                    builder.append(preposition);
                    builder.append(" the ");
                    Self::generate_noun_phrase(parsed, distributions, builder, random);
                }
                'T' => {
                    // Trim trailing space - terminators should abut previous word
                    builder.erase(1);
                    let terminator = distributions.get_terminators().random_value(random);
                    builder.append(terminator);
                }
                _ => panic!("Unknown token '{}'", syntax.chars().nth(i).unwrap()),
            }

            if builder.get_last_char() != ' ' {
                builder.append(" ");
            }
        }
    }

    /// Generate a verb phrase according to TPC-H grammar rules
    fn generate_verb_phrase(
        parsed: &ParsedDistributions,
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RandomInt,
    ) {
        let syntax = distributions.get_verb_phrase().random_value(random);
        let index = parsed.verb_phrases.get_random_index(random);
        let tokens = parsed.verb_phrases.get_tokens(index);

        let max_length = syntax.len();
        for (i, &token) in tokens.iter().enumerate() {
            // Pick a random word
            let source = match token {
                'D' => distributions.get_adverbs(),
                'V' => distributions.get_verbs(),
                'X' => distributions.get_auxiliaries(),
                _ => panic!("Unknown token '{}'", token),
            };

            let word = source.random_value(random);
            builder.append(word);

            // Get bonus text (if any) from the parsed distribution
            let bonus_text = parsed.verb_phrases.get_bonus_text(index);
            builder.append(bonus_text);

            // Add a space
            builder.append(" ");
        }
    }

    /// Generate a noun phrase according to TPC-H grammar rules
    fn generate_noun_phrase(
        parsed: &ParsedDistributions,
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RandomInt,
    ) {
        let syntax = distributions.get_noun_phrase().random_value(random);
        let index = parsed.noun_phrases.get_random_index(random);
        let tokens = parsed.noun_phrases.get_tokens(index);

        let max_length = syntax.len();
        for (i, &token) in tokens.iter().enumerate() {
            // Pick a random word
            let source = match token {
                'A' => distributions.get_articles(),
                'J' => distributions.get_adjectives(),
                'D' => distributions.get_adverbs(),
                'N' => distributions.get_nouns(),
                _ => panic!("Unknown token '{}'", token),
            };

            let word = source.random_value(random);
            builder.append(word);

            // Get bonus text (if any) from the parsed distribution
            let bonus_text = parsed.noun_phrases.get_bonus_text(index);
            builder.append(bonus_text);

            // Add a space
            builder.append(" ");
        }
    }
}

/// Helper struct for efficiently building the text pool
struct ByteArrayBuilder {
    bytes: Vec<u8>,
    length: usize,
}

impl ByteArrayBuilder {
    fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
            length: 0,
        }
    }

    fn append(&mut self, string: &str) {
        let bytes = string.as_bytes();
        self.bytes[self.length..self.length + bytes.len()].copy_from_slice(bytes);
        self.length += bytes.len();
    }

    fn erase(&mut self, count: usize) {
        assert!(self.length >= count, "Not enough bytes to erase");
        self.length -= count;
    }

    fn length(&self) -> usize {
        self.length
    }

    fn get_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    fn get_last_char(&self) -> char {
        self.bytes[self.length - 1] as char
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::distribution::DISTS_SEED;

/// Provides a shared TextPool instance
pub struct TextPoolSupplier {
    text_pool: Mutex<Option<Arc<TextPool>>>,
    init: Once,
}

impl TextPoolSupplier {
    /// Create a new TextPoolSupplier
    pub fn new() -> Self {
        Self {
            text_pool: Mutex::new(None),
            init: Once::new(),
        }
    }

    /// Get the shared TextPool, creating it if needed
    pub fn get(&self) -> Arc<TextPool> {
        // Fast path - check if pool already exists
        {
            let guard = self.text_pool.lock().unwrap();
            if let Some(pool) = guard.as_ref() {
                return pool.clone();
            }
        }

        // Slow path - initialize pool if not exists
        self.init.call_once(|| {
            let pool = Arc::new(TextPool::create_default_text_pool(
                &Distributions::get_default_distributions(),
            ));

            let mut guard = self.text_pool.lock().unwrap();
            *guard = Some(pool);
        });

        // Return the pool
        self.text_pool.lock().unwrap().as_ref().unwrap().clone()
    }
}

use std::sync::OnceLock;

static DEFAULT_TEXT_POOL_SUPPLIER: OnceLock<TextPoolSupplier> = OnceLock::new();

pub fn get_default_text_pool() -> &'static TextPoolSupplier {
    DEFAULT_TEXT_POOL_SUPPLIER.get_or_init(|| TextPoolSupplier::new())
}

/// Generates random text of specified length ranges from the TextPool
pub struct RandomText {
    text_pool: Arc<TextPool>,
    min_length: usize,
    max_length: usize,
    rng: RandomInt,
}

impl RandomText {
    /// Create a new RandomText generator
    pub fn new(seed: i64, text_pool: Arc<TextPool>, average_length: f64) -> Self {
        Self::new_with_expected_row_count(seed, text_pool, average_length, 1)
    }

    /// Create a new RandomText generator with expected row count
    pub fn new_with_expected_row_count(
        seed: i64,
        text_pool: Arc<TextPool>,
        average_length: f64,
        expected_row_count: usize,
    ) -> Self {
        const LOW_LENGTH_MULTIPLIER: f64 = 0.4;
        const HIGH_LENGTH_MULTIPLIER: f64 = 1.6;

        let min_length = (average_length * LOW_LENGTH_MULTIPLIER) as usize;
        let max_length = (average_length * HIGH_LENGTH_MULTIPLIER) as usize;

        Self {
            text_pool,
            min_length,
            max_length,
            rng: RandomInt::new(seed, expected_row_count * 2),
        }
    }

    /// Get the next random text value
    pub fn next_value(&mut self) -> String {
        let offset =
            self.rng
                .next_int(0, (self.text_pool.size() - self.max_length) as i32) as usize;
        let length = self
            .rng
            .next_int(self.min_length as i32, self.max_length as i32) as usize;

        self.text_pool.get_text(offset, offset + length)
    }

    /// Mark row as finished
    pub fn row_finished(&mut self) {
        self.rng.row_finished();
    }

    /// Advance rows
    pub fn advance_rows(&mut self, row_count: usize) {
        self.rng.advance_rows(row_count);
    }
}

/// Generates random alphanumeric strings
pub struct RandomAlphaNumeric {
    inner: AbstractRandomInt,
    min_length: usize,
    max_length: usize,
}

impl RandomAlphaNumeric {
    // Characters allowed in alphanumeric strings
    const ALPHA_NUMERIC: &'static [u8] =
        b"0123456789abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ,";

    // Length multipliers from TPC-H spec
    const LOW_LENGTH_MULTIPLIER: f64 = 0.4;
    const HIGH_LENGTH_MULTIPLIER: f64 = 1.6;

    // Usage count per row
    const USAGE_PER_ROW: usize = 9;

    pub fn new(seed: i64, average_length: usize) -> Self {
        Self::new_with_expected_row_count(seed, average_length, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        average_length: usize,
        expected_row_count: usize,
    ) -> Self {
        let min_length = (average_length as f64 * Self::LOW_LENGTH_MULTIPLIER) as usize;
        let max_length = (average_length as f64 * Self::HIGH_LENGTH_MULTIPLIER) as usize;

        Self {
            inner: AbstractRandomInt::new(seed, Self::USAGE_PER_ROW * expected_row_count),
            min_length,
            max_length,
        }
    }

    pub fn next_value(&mut self) -> String {
        let length = self
            .inner
            .next_int(self.min_length as i32, self.max_length as i32) as usize;
        let mut buffer = vec![0u8; length];

        let mut char_index = 0;
        for i in 0..length {
            if i % 5 == 0 {
                char_index = self.inner.next_int(0, i32::MAX) as i64;
            }

            let char_pos = (char_index & 0x3f) as usize;
            buffer[i] = Self::ALPHA_NUMERIC[char_pos];
            char_index >>= 6;
        }

        // This is safe because ALPHA_NUMERIC contains only valid ASCII
        String::from_utf8(buffer).unwrap()
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }
}
/// Generates phone numbers according to TPC-H spec
pub struct RandomPhoneNumber {
    inner: AbstractRandomInt,
}

impl RandomPhoneNumber {
    // Maximum number of nations in TPC-H
    const NATIONS_MAX: i32 = 90;

    pub fn new(seed: i64) -> Self {
        Self::new_with_expected_row_count(seed, 1)
    }

    pub fn new_with_expected_row_count(seed: i64, expected_row_count: usize) -> Self {
        Self {
            inner: AbstractRandomInt::new(seed, 3 * expected_row_count),
        }
    }

    pub fn next_value(&mut self, nation_key: i64) -> String {
        let country_code = 10 + (nation_key % Self::NATIONS_MAX as i64);
        let local1 = self.inner.next_int(100, 999);
        let local2 = self.inner.next_int(100, 999);
        let local3 = self.inner.next_int(1000, 9999);

        format!(
            "{}-{}-{}-{}",
            Self::pad_with_zeros(country_code, 2),
            Self::pad_with_zeros(local1 as i64, 3),
            Self::pad_with_zeros(local2 as i64, 3),
            Self::pad_with_zeros(local3 as i64, 4)
        )
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }

    // Helper to pad numbers with zeros
    fn pad_with_zeros(value: i64, length: usize) -> String {
        format!("{:0width$}", value, width = length)
    }
}

/// Generates sequences of random strings from a distribution
pub struct RandomStringSequence {
    inner: AbstractRandomInt,
    count: usize,
    distribution: Distribution,
}

impl RandomStringSequence {
    pub fn new(seed: i64, count: usize, distribution: Distribution) -> Self {
        Self::new_with_expected_row_count(seed, count, distribution, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        count: usize,
        distribution: Distribution,
        expected_row_count: usize,
    ) -> Self {
        Self {
            inner: AbstractRandomInt::new(seed, distribution.size() * expected_row_count),
            count,
            distribution,
        }
    }

    pub fn next_value(&mut self) -> String {
        let distribution_size = self.distribution.size();
        assert!(
            self.count < distribution_size,
            "Count must be less than distribution size"
        );

        // Get all values from the distribution
        let mut values: Vec<String> = (0..distribution_size)
            .map(|i| self.distribution.get_value(i).to_string())
            .collect();

        // Randomize first 'count' elements
        for current_position in 0..self.count {
            // Pick a random position to swap with
            let swap_position = self
                .inner
                .next_int(current_position as i32, (distribution_size - 1) as i32)
                as usize;

            // Swap the elements
            values.swap(current_position, swap_position);
        }

        // Join the first 'count' values with spaces
        values[0..self.count].join(" ")
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }
}
/// Selects random strings from a distribution
pub struct RandomString {
    inner: RandomInt,
    distribution: Distribution,
}

impl RandomString {
    pub fn new(seed: i64, distribution: Distribution) -> Self {
        Self::new_with_expected_row_count(seed, distribution, 1)
    }

    pub fn new_with_expected_row_count(
        seed: i64,
        distribution: Distribution,
        expected_row_count: usize,
    ) -> Self {
        Self {
            inner: RandomInt::new(seed, expected_row_count),
            distribution,
        }
    }

    pub fn next_value(&mut self) -> String {
        self.distribution.random_value(&mut self.inner).to_string()
    }

    pub fn row_finished(&mut self) {
        self.inner.row_finished();
    }

    pub fn advance_rows(&mut self, row_count: usize) {
        self.inner.advance_rows(row_count);
    }
}

/// This module defines the data structures representing the TPC-H tables
///
/// The REGION table
#[derive(Debug, Clone)]
pub struct Region {
    /// Primary key (0-4)
    pub r_regionkey: i64,
    /// Region name (AFRICA, AMERICA, ASIA, EUROPE, MIDDLE EAST)
    pub r_name: String,
    /// Variable length comment
    pub r_comment: String,
}

/// The NATION table
#[derive(Debug, Clone)]
pub struct Nation {
    /// Primary key (0-24)
    pub n_nationkey: i64,
    /// Nation name
    pub n_name: String,
    /// Foreign key to REGION
    pub n_regionkey: i64,
    /// Variable length comment
    pub n_comment: String,
}

/// The SUPPLIER table
pub struct Supplier {
    /// Primary key
    pub s_suppkey: i64,
    /// Supplier name
    pub s_name: String,
    /// Supplier address
    pub s_address: String,
    /// Foreign key to NATION
    pub s_nationkey: i64,
    /// Supplier phone number
    pub s_phone: String,
    /// Supplier account balance
    pub s_acctbal: f64,
    /// Variable length comment
    pub s_comment: String,
}

/// The PART table
pub struct Part {
    /// Primary key
    pub p_partkey: i64,
    /// Part name
    pub p_name: String,
    /// Part manufacturer
    pub p_mfgr: String,
    /// Part brand
    pub p_brand: String,
    /// Part type
    pub p_type: String,
    /// Part size
    pub p_size: i32,
    /// Part container
    pub p_container: String,
    /// Part retail price
    pub p_retailprice: f64,
    /// Variable length comment
    pub p_comment: String,
}

/// The PARTSUPP table
pub struct PartSupp {
    /// Primary key, foreign key to PART
    pub ps_partkey: i64,
    /// Primary key, foreign key to SUPPLIER
    pub ps_suppkey: i64,
    /// Available quantity
    pub ps_availqty: i32,
    /// Supplier cost
    pub ps_supplycost: f64,
    /// Variable length comment
    pub ps_comment: String,
}

/// The CUSTOMER table
pub struct Customer {
    /// Primary key
    pub c_custkey: i64,
    /// Customer name
    pub c_name: String,
    /// Customer address
    pub c_address: String,
    /// Foreign key to NATION
    pub c_nationkey: i64,
    /// Customer phone number
    pub c_phone: String,
    /// Customer account balance
    pub c_acctbal: f64,
    /// Customer market segment
    pub c_mktsegment: String,
    /// Variable length comment
    pub c_comment: String,
}

/// The ORDERS table
pub struct Order {
    /// Primary key
    pub o_orderkey: i64,
    /// Foreign key to CUSTOMER
    pub o_custkey: i64,
    /// Order status (F=final, O=open, P=pending)
    pub o_orderstatus: char,
    /// Order total price
    pub o_totalprice: f64,
    /// Order date
    pub o_orderdate: String, // Could use a date type instead
    /// Order priority
    pub o_orderpriority: String,
    /// Clerk who processed the order
    pub o_clerk: String,
    /// Order shipping priority
    pub o_shippriority: i32,
    /// Variable length comment
    pub o_comment: String,
}

/// The LINEITEM table
pub struct LineItem {
    /// Foreign key to ORDERS
    pub l_orderkey: i64,
    /// Foreign key to PART
    pub l_partkey: i64,
    /// Foreign key to SUPPLIER
    pub l_suppkey: i64,
    /// Line item number within order
    pub l_linenumber: i32,
    /// Quantity ordered
    pub l_quantity: f64,
    /// Extended price (l_quantity * p_retailprice)
    pub l_extendedprice: f64,
    /// Discount percentage
    pub l_discount: f64,
    /// Tax percentage
    pub l_tax: f64,
    /// Return flag (R=returned, A=accepted, null=pending)
    pub l_returnflag: String,
    /// Line status (O=ordered, F=fulfilled)
    pub l_linestatus: String,
    /// Date shipped
    pub l_shipdate: String, // Could use a date type instead
    /// Date committed to ship
    pub l_commitdate: String, // Could use a date type instead
    /// Date received
    pub l_receiptdate: String, // Could use a date type instead
    /// Shipping instructions
    pub l_shipinstruct: String,
    /// Shipping mode
    pub l_shipmode: String,
    /// Variable length comment
    pub l_comment: String,
}

/// Generator for the REGION table
pub struct RegionGenerator {
    text_pool: Arc<TextPool>,
}

impl RegionGenerator {
    /// Comment length specified in TPC-H
    const COMMENT_AVERAGE_LENGTH: usize = 72;

    /// Create a new RegionGenerator
    pub fn new(text_pool: Arc<TextPool>) -> Self {
        Self { text_pool }
    }

    /// Generate all regions
    pub fn generate(&self) -> Vec<Region> {
        // The 5 fixed regions from TPC-H spec section 4.2.3
        let regions = [
            (0, "AFRICA"),
            (1, "AMERICA"),
            (2, "ASIA"),
            (3, "EUROPE"),
            (4, "MIDDLE EAST"),
        ];

        let mut comment_gen = RandomText::new(
            1500869201,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );

        let mut result = Vec::with_capacity(regions.len());

        for (index, (region_key, name)) in regions.iter().enumerate() {
            let comment = comment_gen.next_value();

            result.push(Region {
                r_regionkey: *region_key,
                r_name: name.to_string(),
                r_comment: comment,
            });

            comment_gen.row_finished();
        }

        result
    }
}

/// Generator for the NATION table
pub struct NationGenerator {
    text_pool: Arc<TextPool>,
}

impl NationGenerator {
    /// Comment length specified in TPC-H
    const COMMENT_AVERAGE_LENGTH: usize = 72;

    /// Create a new NationGenerator
    pub fn new(text_pool: Arc<TextPool>) -> Self {
        Self { text_pool }
    }

    /// Generate all nations
    pub fn generate(&self) -> Vec<Nation> {
        // The 25 fixed nations from TPC-H spec section 4.2.3
        let nations = [
            (0, "ALGERIA", 0),
            (1, "ARGENTINA", 1),
            (2, "BRAZIL", 1),
            (3, "CANADA", 1),
            (4, "EGYPT", 4),
            (5, "ETHIOPIA", 0),
            (6, "FRANCE", 3),
            (7, "GERMANY", 3),
            (8, "INDIA", 2),
            (9, "INDONESIA", 2),
            (10, "IRAN", 4),
            (11, "IRAQ", 4),
            (12, "JAPAN", 2),
            (13, "JORDAN", 4),
            (14, "KENYA", 0),
            (15, "MOROCCO", 0),
            (16, "MOZAMBIQUE", 0),
            (17, "PERU", 1),
            (18, "CHINA", 2),
            (19, "ROMANIA", 3),
            (20, "SAUDI ARABIA", 4),
            (21, "VIETNAM", 2),
            (22, "RUSSIA", 3),
            (23, "UNITED KINGDOM", 3),
            (24, "UNITED STATES", 1),
        ];

        let mut comment_gen = RandomText::new(
            606179079,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );

        let mut result = Vec::with_capacity(nations.len());

        for (index, (nation_key, name, region_key)) in nations.iter().enumerate() {
            let comment = comment_gen.next_value();

            result.push(Nation {
                n_nationkey: *nation_key,
                n_name: name.to_string(),
                n_regionkey: *region_key,
                n_comment: comment,
            });

            comment_gen.row_finished();
        }

        result
    }
}

/// Generator for the SUPPLIER table
pub struct SupplierGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
}

impl SupplierGenerator {
    /// Base scale factor for suppliers
    const SCALE_BASE: usize = 10_000;

    /// Parameter ranges from TPC-H spec
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: usize = 25;
    const COMMENT_AVERAGE_LENGTH: usize = 63;

    /// BBB comment parameters for suppliers
    const BBB_BASE_TEXT: &'static str = "Customer ";
    const BBB_COMPLAINT_TEXT: &'static str = "Complaints";
    const BBB_RECOMMEND_TEXT: &'static str = "Recommends";
    const BBB_COMMENT_LENGTH: usize = Self::BBB_BASE_TEXT.len() + Self::BBB_COMPLAINT_TEXT.len();
    const BBB_COMMENTS_PER_SCALE_BASE: i32 = 10;
    const BBB_COMPLAINT_PERCENT: i32 = 50;

    /// Create a new SupplierGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (total_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate suppliers for this part
    pub fn generate(&self) -> Vec<Supplier> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize random generators
        let mut address_random = RandomAlphaNumeric::new(706178559, Self::ADDRESS_AVERAGE_LENGTH);
        let mut nation_key_random = RandomBoundedInt::new(110356601, 0, 24);
        let mut phone_random = RandomPhoneNumber::new(884434366);
        let mut account_balance_random = RandomBoundedInt::new(
            962338209,
            Self::ACCOUNT_BALANCE_MIN,
            Self::ACCOUNT_BALANCE_MAX,
        );
        let mut comment_random = RandomText::new(
            1341315363,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );
        let mut bbb_comment_random = RandomBoundedInt::new(202794285, 1, Self::SCALE_BASE as i32);
        let mut bbb_junk_random =
            RandomBoundedInt::new(263032577, 0, Self::COMMENT_AVERAGE_LENGTH as i32);
        let mut bbb_offset_random =
            RandomBoundedInt::new(715851524, 0, Self::COMMENT_AVERAGE_LENGTH as i32);
        let mut bbb_type_random = RandomBoundedInt::new(753643799, 0, 100);

        // Advance random generators to correct starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);
        bbb_comment_random.advance_rows(start_index);
        bbb_junk_random.advance_rows(start_index);
        bbb_offset_random.advance_rows(start_index);
        bbb_type_random.advance_rows(start_index);

        let mut suppliers = Vec::with_capacity(row_count);

        for i in 0..row_count {
            let supplier_key = start_index + i + 1;

            // Generate basic supplier data
            let mut comment = comment_random.next_value();

            // Add supplier complaints or commendation to the comment
            let bbb_comment_value = bbb_comment_random.next_value();
            if bbb_comment_value <= Self::BBB_COMMENTS_PER_SCALE_BASE {
                // Select random place for BBB comment
                let noise = bbb_junk_random.next_value() as usize;
                let offset = bbb_offset_random.next_value() as usize;
                let max_offset = comment
                    .len()
                    .saturating_sub(Self::BBB_COMMENT_LENGTH + noise);
                let actual_offset = offset.min(max_offset);

                // Select complaint or recommendation
                let bbb_type = if bbb_type_random.next_value() < Self::BBB_COMPLAINT_PERCENT {
                    Self::BBB_COMPLAINT_TEXT
                } else {
                    Self::BBB_RECOMMEND_TEXT
                };

                // Create modified comment with BBB text
                let mut buffer = comment.clone();
                let base_offset = actual_offset;
                let type_offset = base_offset + Self::BBB_BASE_TEXT.len() + noise;

                // Replace text at positions
                buffer.replace_range(
                    base_offset
                        ..base_offset + Self::BBB_BASE_TEXT.len().min(buffer.len() - base_offset),
                    Self::BBB_BASE_TEXT,
                );

                if type_offset < buffer.len() {
                    let type_end = type_offset + bbb_type.len().min(buffer.len() - type_offset);
                    buffer.replace_range(type_offset..type_end, bbb_type);
                }

                comment = buffer;
            }

            let nation_key = nation_key_random.next_value() as i64;

            suppliers.push(Supplier {
                s_suppkey: supplier_key as i64,
                s_name: format!("Supplier#{}", pad_with_zeros(supplier_key as i64, 9)),
                s_address: address_random.next_value(),
                s_nationkey: nation_key,
                s_phone: phone_random.next_value(nation_key),
                s_acctbal: account_balance_random.next_value() as f64 / 100.0,
                s_comment: comment,
            });

            // Mark row as finished for all generators
            address_random.row_finished();
            nation_key_random.row_finished();
            phone_random.row_finished();
            account_balance_random.row_finished();
            comment_random.row_finished();
            bbb_comment_random.row_finished();
            bbb_junk_random.row_finished();
            bbb_offset_random.row_finished();
            bbb_type_random.row_finished();
        }

        suppliers
    }
}

// Helper function for padding zeros
fn pad_with_zeros(value: i64, width: usize) -> String {
    format!("{:0width$}", value, width = width)
}

/// Generator for the PART table
pub struct PartGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
    part_colors: Distribution,
    part_types: Distribution,
    part_containers: Distribution,
}

impl PartGenerator {
    /// Base scale factor for parts
    const SCALE_BASE: usize = 200_000;

    /// Parameter ranges from TPC-H spec
    const NAME_WORDS: usize = 5;
    const MANUFACTURER_MIN: i32 = 1;
    const MANUFACTURER_MAX: i32 = 5;
    const BRAND_MIN: i32 = 1;
    const BRAND_MAX: i32 = 5;
    const SIZE_MIN: i32 = 1;
    const SIZE_MAX: i32 = 50;
    const COMMENT_AVERAGE_LENGTH: usize = 14;

    /// Create a new PartGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
        part_colors: Distribution,
        part_types: Distribution,
        part_containers: Distribution,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
            part_colors,
            part_types,
            part_containers,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (total_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate parts for this part
    pub fn generate(&self) -> Vec<Part> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize random generators
        let mut name_random =
            RandomStringSequence::new(709314158, Self::NAME_WORDS, self.part_colors.clone());
        let mut manufacturer_random =
            RandomBoundedInt::new(1, Self::MANUFACTURER_MIN, Self::MANUFACTURER_MAX);
        let mut brand_random = RandomBoundedInt::new(46831694, Self::BRAND_MIN, Self::BRAND_MAX);
        let mut type_random = RandomString::new(1841581359, self.part_types.clone());
        let mut size_random = RandomBoundedInt::new(1193163244, Self::SIZE_MIN, Self::SIZE_MAX);
        let mut container_random = RandomString::new(727633698, self.part_containers.clone());
        let mut comment_random = RandomText::new(
            804159733,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance random generators to correct starting position
        name_random.advance_rows(start_index);
        manufacturer_random.advance_rows(start_index);
        brand_random.advance_rows(start_index);
        type_random.advance_rows(start_index);
        size_random.advance_rows(start_index);
        container_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        let mut parts = Vec::with_capacity(row_count);

        for i in 0..row_count {
            let part_key = start_index + i + 1;

            // Generate part name from random colors
            let name = name_random.next_value();

            // Generate manufacturer and brand
            let manufacturer = manufacturer_random.next_value();
            let brand = manufacturer * 10 + brand_random.next_value();

            // Calculate retail price as per spec
            let retail_price = calculate_part_price(part_key as i64);

            parts.push(Part {
                p_partkey: part_key as i64,
                p_name: name,
                p_mfgr: format!("Manufacturer#{}", manufacturer),
                p_brand: format!("Brand#{}", brand),
                p_type: type_random.next_value(),
                p_size: size_random.next_value(),
                p_container: container_random.next_value(),
                p_retailprice: retail_price,
                p_comment: comment_random.next_value(),
            });

            // Mark row as finished for all generators
            name_random.row_finished();
            manufacturer_random.row_finished();
            brand_random.row_finished();
            type_random.row_finished();
            size_random.row_finished();
            container_random.row_finished();
            comment_random.row_finished();
        }

        parts
    }
}

/// Calculate part price according to TPC-H spec
pub fn calculate_part_price(part_key: i64) -> f64 {
    let mut price = 90000;

    // Limit contribution to $200
    price += ((part_key / 10) % 20001) as i64;
    price += (part_key % 1000) * 100;

    price as f64 / 100.0
}

/// Generator for the PARTSUPP table
pub struct PartSuppGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
}

impl PartSuppGenerator {
    /// Number of suppliers per part
    const SUPPLIERS_PER_PART: usize = 4;

    /// Parameter ranges from TPC-H spec
    const AVAILABLE_QUANTITY_MIN: i32 = 1;
    const AVAILABLE_QUANTITY_MAX: i32 = 9999;
    const SUPPLY_COST_MIN: i32 = 100;
    const SUPPLY_COST_MAX: i32 = 100000;
    const COMMENT_AVERAGE_LENGTH: usize = 124;

    /// Create a new PartSuppGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let parts_count = (self.scale_factor * PartGenerator::SCALE_BASE as f64) as usize;
        let per_part = parts_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let parts_count = (self.scale_factor * PartGenerator::SCALE_BASE as f64) as usize;
        let per_part = parts_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (parts_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate part-supplier relationships for this part
    pub fn generate(&self) -> Vec<PartSupp> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize random generators
        let mut available_quantity_random = RandomBoundedInt::new_with_expected_row_count(
            1671059989,
            Self::AVAILABLE_QUANTITY_MIN,
            Self::AVAILABLE_QUANTITY_MAX,
            Self::SUPPLIERS_PER_PART,
        );

        let mut supply_cost_random = RandomBoundedInt::new_with_expected_row_count(
            1051288424,
            Self::SUPPLY_COST_MIN,
            Self::SUPPLY_COST_MAX,
            Self::SUPPLIERS_PER_PART,
        );

        let mut comment_random = RandomText::new_with_expected_row_count(
            1961692154,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
            Self::SUPPLIERS_PER_PART,
        );

        // Advance random generators to correct starting position
        available_quantity_random.advance_rows(start_index);
        supply_cost_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        let result_capacity = row_count * Self::SUPPLIERS_PER_PART;
        let mut partsupp = Vec::with_capacity(result_capacity);

        // For each part, generate 4 supplier relationships
        for part_index in 0..row_count {
            let part_key = (start_index + part_index + 1) as i64;

            for supp_number in 0..Self::SUPPLIERS_PER_PART {
                let supp_key =
                    Self::select_part_supplier(part_key, supp_number as i64, self.scale_factor);

                partsupp.push(PartSupp {
                    ps_partkey: part_key,
                    ps_suppkey: supp_key,
                    ps_availqty: available_quantity_random.next_value(),
                    ps_supplycost: supply_cost_random.next_value() as f64 / 100.0,
                    ps_comment: comment_random.next_value(),
                });
            }

            // Mark row as finished for all generators
            available_quantity_random.row_finished();
            supply_cost_random.row_finished();
            comment_random.row_finished();
        }

        partsupp
    }

    /// Selects a supplier for a part according to TPC-H spec
    pub fn select_part_supplier(part_key: i64, supplier_number: i64, scale_factor: f64) -> i64 {
        let supplier_count = (SupplierGenerator::SCALE_BASE as f64 * scale_factor) as i64;

        // Formula from TPC-H specification
        let supplier_index = part_key
            + (supplier_number
                * ((supplier_count / PartSuppGenerator::SUPPLIERS_PER_PART as i64)
                    + ((part_key - 1) / supplier_count)));

        (supplier_index % supplier_count) + 1
    }
}

/// Generator for the CUSTOMER table
pub struct CustomerGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
    market_segments: Distribution,
}

impl CustomerGenerator {
    /// Base scale factor for customers
    const SCALE_BASE: usize = 150_000;

    /// Parameter ranges from TPC-H spec
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: usize = 25;
    const COMMENT_AVERAGE_LENGTH: usize = 73;

    /// Create a new CustomerGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
        market_segments: Distribution,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
            market_segments,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (total_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate customers for this part
    pub fn generate(&self) -> Vec<Customer> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize random generators
        let mut address_random = RandomAlphaNumeric::new(881155353, Self::ADDRESS_AVERAGE_LENGTH);

        let mut nation_key_random = RandomBoundedInt::new(1489529863, 0, 24);

        let mut phone_random = RandomPhoneNumber::new(1521138112);

        let mut account_balance_random = RandomBoundedInt::new(
            298370230,
            Self::ACCOUNT_BALANCE_MIN,
            Self::ACCOUNT_BALANCE_MAX,
        );

        let mut market_segment_random = RandomString::new(1140279430, self.market_segments.clone());

        let mut comment_random = RandomText::new(
            1335826707,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance random generators to correct starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        market_segment_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        let mut customers = Vec::with_capacity(row_count);

        for i in 0..row_count {
            let customer_key = start_index + i + 1;
            let nation_key = nation_key_random.next_value() as i64;

            customers.push(Customer {
                c_custkey: customer_key as i64,
                c_name: format!("Customer#{}", pad_with_zeros(customer_key as i64, 9)),
                c_address: address_random.next_value(),
                c_nationkey: nation_key,
                c_phone: phone_random.next_value(nation_key),
                c_acctbal: account_balance_random.next_value() as f64 / 100.0,
                c_mktsegment: market_segment_random.next_value(),
                c_comment: comment_random.next_value(),
            });

            // Mark row as finished for all generators
            address_random.row_finished();
            nation_key_random.row_finished();
            phone_random.row_finished();
            account_balance_random.row_finished();
            market_segment_random.row_finished();
            comment_random.row_finished();
        }

        customers
    }
}

/// Generator for the ORDERS table
pub struct OrderGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
    order_priorities: Distribution,
}

impl OrderGenerator {
    /// Base scale factor for orders
    const SCALE_BASE: usize = 1_500_000;

    /// Customer mortality rate - portion with no orders
    const CUSTOMER_MORTALITY: usize = 3;

    /// Clerk scale base
    const CLERK_SCALE_BASE: usize = 1000;

    /// Order date range
    const ORDER_DATE_MIN: i32 = 92001; // MIN_GENERATE_DATE from GenerateUtils
    const ORDER_DATE_MAX: i32 = 97996; // Calculated based on TPC-H spec

    /// Line count range
    const LINE_COUNT_MIN: i32 = 1;
    const LINE_COUNT_MAX: i32 = 7;

    /// Comment average length
    const COMMENT_AVERAGE_LENGTH: usize = 49;

    /// Order key sparsity parameters
    const ORDER_KEY_SPARSE_BITS: usize = 2;
    const ORDER_KEY_SPARSE_KEEP: usize = 3;

    /// Create a new OrderGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
        order_priorities: Distribution,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
            order_priorities,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let total_count = (self.scale_factor * Self::SCALE_BASE as f64) as usize;
        let per_part = total_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (total_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate orders for this part
    pub fn generate(&self) -> Vec<Order> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize random generators
        let mut order_date_random = Self::create_order_date_random();
        let mut line_count_random = Self::create_line_count_random();

        let use_64bits = self.scale_factor >= 30000.0;
        let max_customer_key = (CustomerGenerator::SCALE_BASE as f64 * self.scale_factor) as i64;

        let mut customer_key_random =
            RandomBoundedLong::new(851767375, use_64bits, 1, max_customer_key);

        let mut order_priority_random = RandomString::new(591449447, self.order_priorities.clone());

        let clerk_max = (self.scale_factor * Self::CLERK_SCALE_BASE as f64) as i32;
        let clerk_max = if clerk_max < Self::CLERK_SCALE_BASE as i32 {
            Self::CLERK_SCALE_BASE as i32
        } else {
            clerk_max
        };

        let mut clerk_random = RandomBoundedInt::new(1171034773, 1, clerk_max);

        let mut comment_random = RandomText::new(
            276090261,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
        );

        // We need LineItem generators to determine order status and total price
        // For simplicity, we'll use a placeholder approach here
        // In a full implementation, you'd coordinate with LineItemGenerator

        // Advance random generators to correct starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);
        customer_key_random.advance_rows(start_index);
        order_priority_random.advance_rows(start_index);
        clerk_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        let mut orders = Vec::with_capacity(row_count);

        for i in 0..row_count {
            let order_index = start_index + i + 1;
            let order_key = Self::make_order_key(order_index as i64);

            // Generate order date
            let order_date = order_date_random.next_value();
            let order_date_string = epoch_date_to_string(order_date);

            // Generate customer key, taking into account customer mortality rate
            let mut customer_key = customer_key_random.next_value();
            let mut delta = 1;

            while customer_key % Self::CUSTOMER_MORTALITY as i64 == 0 {
                customer_key += delta;
                if customer_key > max_customer_key {
                    customer_key = max_customer_key;
                }
                delta *= -1;
            }

            // In a full implementation, you would calculate order status and total price
            // based on the line items for this order
            // Here we'll use placeholder values
            let order_status = 'O'; // Placeholder
            let total_price = 0.0; // Placeholder

            orders.push(Order {
                o_orderkey: order_key,
                o_custkey: customer_key,
                o_orderstatus: order_status,
                o_totalprice: total_price,
                o_orderdate: order_date_string,
                o_orderpriority: order_priority_random.next_value(),
                o_clerk: format!(
                    "Clerk#{}",
                    pad_with_zeros(clerk_random.next_value() as i64, 9)
                ),
                o_shippriority: 0,
                o_comment: comment_random.next_value(),
            });

            // Mark row as finished for all generators
            order_date_random.row_finished();
            line_count_random.row_finished();
            customer_key_random.row_finished();
            order_priority_random.row_finished();
            clerk_random.row_finished();
            comment_random.row_finished();
        }

        orders
    }

    /// Create random generator for order dates
    fn create_order_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1066728069, Self::ORDER_DATE_MIN, Self::ORDER_DATE_MAX)
    }

    /// Create random generator for line counts
    fn create_line_count_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1434868289, Self::LINE_COUNT_MIN, Self::LINE_COUNT_MAX)
    }

    /// Make an order key according to TPC-H spec
    pub fn make_order_key(order_index: i64) -> i64 {
        let low_bits = order_index & ((1 << Self::ORDER_KEY_SPARSE_KEEP) - 1);

        let mut ok = order_index;
        ok >>= Self::ORDER_KEY_SPARSE_KEEP as i64;
        ok <<= Self::ORDER_KEY_SPARSE_BITS as i64;
        ok <<= Self::ORDER_KEY_SPARSE_KEEP as i64;
        ok += low_bits;

        ok
    }
}

/// Convert epoch date to string format
fn epoch_date_to_string(epoch_date: i32) -> String {
    // This is a placeholder - in a real implementation you'd convert
    // from TPC-H epoch date to a proper date string
    // For now we'll just return a dummy date
    "1995-01-01".to_string()
}

/// Generator for the LINEITEM table
pub struct LineItemGenerator {
    scale_factor: f64,
    part: usize,
    part_count: usize,
    text_pool: Arc<TextPool>,
    ship_instructions: Distribution,
    ship_modes: Distribution,
}

impl LineItemGenerator {
    /// Parameter ranges from TPC-H spec
    const QUANTITY_MIN: i32 = 1;
    const QUANTITY_MAX: i32 = 50;
    const TAX_MIN: i32 = 0;
    const TAX_MAX: i32 = 8;
    const DISCOUNT_MIN: i32 = 0;
    const DISCOUNT_MAX: i32 = 10;
    const PART_KEY_MIN: i32 = 1;

    const SHIP_DATE_MIN: i32 = 1;
    const SHIP_DATE_MAX: i32 = 121;
    const COMMIT_DATE_MIN: i32 = 30;
    const COMMIT_DATE_MAX: i32 = 90;
    const RECEIPT_DATE_MIN: i32 = 1;
    const RECEIPT_DATE_MAX: i32 = 30;

    const ITEM_SHIP_DAYS: i32 = Self::SHIP_DATE_MAX + Self::RECEIPT_DATE_MAX;

    const COMMENT_AVERAGE_LENGTH: usize = 27;

    /// Create a new LineItemGenerator
    pub fn new(
        scale_factor: f64,
        part: usize,
        part_count: usize,
        text_pool: Arc<TextPool>,
        ship_instructions: Distribution,
        ship_modes: Distribution,
    ) -> Self {
        Self {
            scale_factor,
            part,
            part_count,
            text_pool,
            ship_instructions,
            ship_modes,
        }
    }

    /// Calculate start index for this part
    fn start_index(&self) -> usize {
        let orders_count = (self.scale_factor * OrderGenerator::SCALE_BASE as f64) as usize;
        let per_part = orders_count / self.part_count;
        per_part * (self.part - 1)
    }

    /// Calculate row count for this part
    fn row_count(&self) -> usize {
        let orders_count = (self.scale_factor * OrderGenerator::SCALE_BASE as f64) as usize;
        let per_part = orders_count / self.part_count;

        if self.part == self.part_count {
            // Last part gets any remainder
            per_part + (orders_count % self.part_count)
        } else {
            per_part
        }
    }

    /// Generate line items for this part
    pub fn generate(&self) -> Vec<LineItem> {
        let start_index = self.start_index();
        let row_count = self.row_count();

        // Initialize all the random generators
        let mut order_date_random = OrderGenerator::create_order_date_random();
        let mut line_count_random = OrderGenerator::create_line_count_random();

        let mut quantity_random = Self::create_quantity_random();
        let mut discount_random = Self::create_discount_random();
        let mut tax_random = Self::create_tax_random();

        let use_64bits = self.scale_factor >= 30000.0;
        let max_part_key = (PartGenerator::SCALE_BASE as f64 * self.scale_factor) as i64;

        let mut part_key_random = RandomBoundedLong::new_with_expected_row_count(
            1808217256,
            use_64bits,
            Self::PART_KEY_MIN as i64,
            max_part_key,
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut supplier_number_random = RandomBoundedInt::new_with_expected_row_count(
            2095021727,
            0,
            3,
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut ship_date_random = Self::create_ship_date_random();

        let mut commit_date_random = RandomBoundedInt::new_with_expected_row_count(
            904914315,
            Self::COMMIT_DATE_MIN,
            Self::COMMIT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut receipt_date_random = RandomBoundedInt::new_with_expected_row_count(
            373135028,
            Self::RECEIPT_DATE_MIN,
            Self::RECEIPT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut return_flag_random = RandomString::new_with_expected_row_count(
            717419739,
            // This should be a return flags distribution
            self.ship_modes.clone(), // Placeholder
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut ship_instructions_random = RandomString::new_with_expected_row_count(
            1371272478,
            self.ship_instructions.clone(),
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut ship_mode_random = RandomString::new_with_expected_row_count(
            675466456,
            self.ship_modes.clone(),
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        let mut comment_random = RandomText::new_with_expected_row_count(
            1095462486,
            self.text_pool.clone(),
            Self::COMMENT_AVERAGE_LENGTH as f64,
            OrderGenerator::LINE_COUNT_MAX as usize,
        );

        // Advance random generators to correct starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);

        quantity_random.advance_rows(start_index);
        discount_random.advance_rows(start_index);
        tax_random.advance_rows(start_index);

        part_key_random.advance_rows(start_index);
        supplier_number_random.advance_rows(start_index);

        ship_date_random.advance_rows(start_index);
        commit_date_random.advance_rows(start_index);
        receipt_date_random.advance_rows(start_index);

        return_flag_random.advance_rows(start_index);
        ship_instructions_random.advance_rows(start_index);
        ship_mode_random.advance_rows(start_index);

        comment_random.advance_rows(start_index);

        // This is an approximation - the actual number will depend on
        // the random line counts generated
        let estimated_capacity = row_count * 4; // Average of 4 line items per order
        let mut line_items = Vec::with_capacity(estimated_capacity);

        let current_date = 95168; // CURRENT_DATE from GenerateUtils

        // For each order
        for order_index in 0..row_count {
            let order_key = OrderGenerator::make_order_key((start_index + order_index + 1) as i64);

            // Generate order date and line count
            let order_date = order_date_random.next_value();
            let line_count = line_count_random.next_value() as usize;

            // For each line item in the order
            for line_number in 0..line_count {
                // Generate quantity, discount, tax
                let quantity = quantity_random.next_value() as i64;
                let discount = discount_random.next_value();
                let tax = tax_random.next_value();

                // Generate part key and supplier key
                let part_key = part_key_random.next_value();
                let supplier_number = supplier_number_random.next_value() as i64;
                let supplier_key = PartSuppGenerator::select_part_supplier(
                    part_key,
                    supplier_number,
                    self.scale_factor,
                );

                // Calculate extended price based on retail price
                let part_price = calculate_part_price(part_key);
                let extended_price = part_price * quantity as f64;

                // Generate dates
                let ship_date_offset = ship_date_random.next_value();
                let ship_date = order_date + ship_date_offset;

                let commit_date_offset = commit_date_random.next_value();
                let commit_date = order_date + commit_date_offset;

                let receipt_date_offset = receipt_date_random.next_value();
                let receipt_date = ship_date + receipt_date_offset;

                // Determine return flag
                let return_flag = if receipt_date <= current_date {
                    // Should randomly choose between "R" and "A"
                    "R".to_string()
                } else {
                    "N".to_string()
                };

                // Determine line status
                let line_status = if ship_date > current_date {
                    "O".to_string()
                } else {
                    "F".to_string()
                };

                line_items.push(LineItem {
                    l_orderkey: order_key,
                    l_partkey: part_key,
                    l_suppkey: supplier_key,
                    l_linenumber: (line_number + 1) as i32,
                    l_quantity: quantity as f64,
                    l_extendedprice: extended_price,
                    l_discount: discount as f64 / 100.0,
                    l_tax: tax as f64 / 100.0,
                    l_returnflag: return_flag,
                    l_linestatus: line_status,
                    l_shipdate: epoch_date_to_string(ship_date),
                    l_commitdate: epoch_date_to_string(commit_date),
                    l_receiptdate: epoch_date_to_string(receipt_date),
                    l_shipinstruct: ship_instructions_random.next_value(),
                    l_shipmode: ship_mode_random.next_value(),
                    l_comment: comment_random.next_value(),
                });
            }

            // After processing all lines for an order, mark row as finished
            order_date_random.row_finished();
            line_count_random.row_finished();

            quantity_random.row_finished();
            discount_random.row_finished();
            tax_random.row_finished();

            part_key_random.row_finished();
            supplier_number_random.row_finished();

            ship_date_random.row_finished();
            commit_date_random.row_finished();
            receipt_date_random.row_finished();

            return_flag_random.row_finished();
            ship_instructions_random.row_finished();
            ship_mode_random.row_finished();

            comment_random.row_finished();
        }

        line_items
    }

    /// Create random generator for quantities
    pub fn create_quantity_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_expected_row_count(
            209208115,
            Self::QUANTITY_MIN,
            Self::QUANTITY_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        )
    }

    /// Create random generator for discounts
    pub fn create_discount_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_expected_row_count(
            554590007,
            Self::DISCOUNT_MIN,
            Self::DISCOUNT_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        )
    }

    /// Create random generator for taxes
    pub fn create_tax_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_expected_row_count(
            721958466,
            Self::TAX_MIN,
            Self::TAX_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        )
    }

    /// Create random generator for part keys
    pub fn create_part_key_random(scale_factor: f64) -> RandomBoundedLong {
        RandomBoundedLong::new_with_expected_row_count(
            1808217256,
            scale_factor >= 30000.0,
            Self::PART_KEY_MIN as i64,
            PartGenerator::SCALE_BASE as i64 * scale_factor as i64,
            OrderGenerator::LINE_COUNT_MAX as usize,
        )
    }

    /// Create random generator for part ship dates.
    pub fn create_ship_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_expected_row_count(
            1769349045,
            Self::SHIP_DATE_MIN,
            Self::SHIP_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX as usize,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_generate_regions_table() {
        let generator = RegionGenerator::new(get_default_text_pool().get());
        let regions = generator.generate();
        assert_eq!(regions.len(), 5);
        println!("{:?}", regions);
    }

    #[test]
    fn can_generate_nations_table() {
        let generator = NationGenerator::new(get_default_text_pool().get());
        let nations = generator.generate();
        assert_eq!(nations.len(), 25);
        println!("{:?}", nations);
    }
}
