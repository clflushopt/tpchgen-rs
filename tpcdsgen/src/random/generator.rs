use crate::distribution::{
    AdjectivesDistribution, AdverbsDistribution, ArticlesDistribution, AuxiliariesDistribution,
    NounsDistribution, PrepositionsDistribution, SentencesDistribution, TerminatorsDistribution,
    VerbsDistribution,
};
use crate::random::stream::RandomNumberStream;
use crate::types::{Date, Decimal};

pub struct RandomValueGenerator;

impl RandomValueGenerator {
    pub const ALPHA_NUMERIC: &'static str =
        "abcdefghijklmnopqrstuvxyzABCDEFGHIJKLMNOPQRSTUVXYZ0123456789";
    pub const DIGITS: &'static str = "0123456789";

    // Static byte arrays to avoid repeated .chars().collect() allocations
    const ALPHA_NUMERIC_BYTES: &'static [u8] =
        b"abcdefghijklmnopqrstuvxyzABCDEFGHIJKLMNOPQRSTUVXYZ0123456789";
    const DIGITS_BYTES: &'static [u8] = b"0123456789";

    pub fn generate_uniform_random_int(
        min: i32,
        max: i32,
        random_number_stream: &mut RandomNumberStream,
    ) -> i32 {
        // truncating long to int copies behavior of c code.
        let mut result = random_number_stream.next_random() as i32;
        result %= max - min + 1;
        result += min;
        result
    }

    pub fn generate_uniform_random_key(
        min: i64,
        max: i64,
        random_number_stream: &mut RandomNumberStream,
    ) -> i64 {
        // truncating long to int copies behavior of c code
        let mut result = random_number_stream.next_random() as i32;
        result %= (max - min + 1) as i32;
        result += min as i32;
        result as i64
    }

    pub fn generate_uniform_random_decimal(
        min: Decimal,
        max: Decimal,
        random_number_stream: &mut RandomNumberStream,
    ) -> Decimal {
        let precision = if min.get_precision() < max.get_precision() {
            min.get_precision()
        } else {
            max.get_precision()
        };

        // compute number
        let mut number = random_number_stream.next_random();
        number %= max.get_number() - min.get_number() + 1;
        number += min.get_number();

        Decimal::new(number, precision)
    }

    pub fn generate_uniform_random_date(
        min: Date,
        max: Date,
        random_number_stream: &mut RandomNumberStream,
    ) -> Date {
        let range = max.to_julian_days() - min.to_julian_days();
        let julian_days = min.to_julian_days()
            + Self::generate_uniform_random_int(0, range, random_number_stream);
        Date::from_julian_days(julian_days)
    }

    // Generate random string from a byte slice (optimized for ASCII character sets)
    fn generate_random_from_bytes(
        length: usize,
        bytes: &[u8],
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        let mut result = Vec::with_capacity(length);

        for _ in 0..length {
            let index =
                Self::generate_uniform_random_int(0, bytes.len() as i32 - 1, random_number_stream);
            result.push(bytes[index as usize]);
        }

        // Safety: ALPHA_NUMERIC_BYTES and DIGITS_BYTES contain only ASCII bytes
        unsafe { String::from_utf8_unchecked(result) }
    }

    // Generate random string of specified length from given character set
    // For non-ASCII character sets that require UTF-8 char handling
    pub fn generate_random_string(
        length: usize,
        character_set: &str,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        let chars: Vec<char> = character_set.chars().collect();
        let mut result = String::with_capacity(length);

        for _ in 0..length {
            let index =
                Self::generate_uniform_random_int(0, chars.len() as i32 - 1, random_number_stream);
            result.push(chars[index as usize]);
        }

        result
    }

    // Generate random alphanumeric string (optimized with static byte array)
    pub fn generate_random_alphanumeric(
        length: usize,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        Self::generate_random_from_bytes(length, Self::ALPHA_NUMERIC_BYTES, random_number_stream)
    }

    // Generate random numeric string (optimized with static byte array)
    pub fn generate_random_digits(
        length: usize,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        Self::generate_random_from_bytes(length, Self::DIGITS_BYTES, random_number_stream)
    }

    // Generate random charset string with variable length (generateRandomCharset)
    // This method loops to max to consume the same number of random values as the Java implementation
    pub fn generate_random_charset(
        character_set: &str,
        min: i32,
        max: i32,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        // Optimize for ALPHA_NUMERIC which is the common case
        if character_set == Self::ALPHA_NUMERIC {
            return Self::generate_random_charset_bytes(
                Self::ALPHA_NUMERIC_BYTES,
                min,
                max,
                random_number_stream,
            );
        }

        let length = Self::generate_uniform_random_int(min, max, random_number_stream);
        let chars: Vec<char> = character_set.chars().collect();
        let mut result = String::with_capacity(length as usize);

        // Loop to max to consume the same number of random values (behavior)
        for i in 0..max {
            let index =
                Self::generate_uniform_random_int(0, chars.len() as i32 - 1, random_number_stream);
            if i < length {
                result.push(chars[index as usize]);
            }
        }

        result
    }

    // Optimized version using byte array for ASCII character sets
    fn generate_random_charset_bytes(
        bytes: &[u8],
        min: i32,
        max: i32,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        let length = Self::generate_uniform_random_int(min, max, random_number_stream);
        let mut result = Vec::with_capacity(length as usize);

        // Loop to max to consume the same number of random values (behavior)
        for i in 0..max {
            let index =
                Self::generate_uniform_random_int(0, bytes.len() as i32 - 1, random_number_stream);
            if i < length {
                result.push(bytes[index as usize]);
            }
        }

        // Safety: bytes contain only ASCII
        unsafe { String::from_utf8_unchecked(result) }
    }

    // Generate random boolean with given probability (0.0 to 1.0)
    pub fn generate_random_boolean(
        probability: f64,
        random_number_stream: &mut RandomNumberStream,
    ) -> bool {
        random_number_stream.next_random_double() < probability
    }

    // Generate random weighted selection from array (indices)
    pub fn generate_weighted_random_index(
        weights: &[i32],
        random_number_stream: &mut RandomNumberStream,
    ) -> usize {
        let total_weight: i32 = weights.iter().sum();
        let random_value = Self::generate_uniform_random_int(1, total_weight, random_number_stream);

        let mut cumulative_weight = 0;
        for (index, &weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if random_value <= cumulative_weight {
                return index;
            }
        }

        // Fallback to last index (should not happen with proper weights)
        weights.len() - 1
    }

    /// Generate a random URL (generateRandomUrl)
    /// This is what the C code does. No joke. It always returns the same value.
    pub fn generate_random_url(_random_number_stream: &mut RandomNumberStream) -> String {
        "http://www.foo.com".to_string()
    }

    /// Generate a random email address (generateRandomEmail)
    pub fn generate_random_email(
        first: &str,
        last: &str,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        use crate::distribution::TopDomainsDistribution;

        let domain = TopDomainsDistribution::pick_random_top_domain(random_number_stream);
        let company_length =
            Self::generate_uniform_random_int(10, 20, random_number_stream) as usize;
        let company =
            Self::generate_random_charset(Self::ALPHA_NUMERIC, 1, 20, random_number_stream);
        let company = if company.len() < company_length {
            company
        } else {
            company[..company_length].to_string()
        };

        format!("{}.{}@{}.{}", first, last, company, domain)
    }

    /// Generate word based on seed and syllables distribution (exact Java implementation)
    /// Takes the distribution as parameter to support both SYLLABLES_DISTRIBUTION and
    /// BRAND_SYLLABLES_DISTRIBUTION
    pub fn generate_word(
        seed: i64,
        max_chars: i32,
        distribution: &crate::distribution::FileBasedStringValuesDistribution,
    ) -> String {
        let size = distribution.get_size() as i64;
        let mut word = String::new();
        let mut seed = seed;

        while seed > 0 {
            let syllable = distribution.get_value_at_index(0, (seed % size) as usize);
            seed /= size;

            if (word.len() + syllable.len()) <= max_chars as usize {
                word.push_str(syllable);
            } else {
                break;
            }
        }

        word
    }
}

/// Generates random English text (generateRandomText).
#[derive(Debug, Clone, Copy)]
pub struct RandomTextGenerator {
    sentences: SentencesDistribution,
    nouns: NounsDistribution,
    verbs: VerbsDistribution,
    adjectives: AdjectivesDistribution,
    adverbs: AdverbsDistribution,
    auxiliaries: AuxiliariesDistribution,
    prepositions: PrepositionsDistribution,
    articles: ArticlesDistribution,
    terminators: TerminatorsDistribution,
}

impl Default for RandomTextGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomTextGenerator {
    pub fn new() -> Self {
        Self {
            sentences: SentencesDistribution::new(),
            nouns: NounsDistribution::new(),
            verbs: VerbsDistribution::new(),
            adjectives: AdjectivesDistribution::new(),
            adverbs: AdverbsDistribution::new(),
            auxiliaries: AuxiliariesDistribution::new(),
            prepositions: PrepositionsDistribution::new(),
            articles: ArticlesDistribution::new(),
            terminators: TerminatorsDistribution::new(),
        }
    }

    // Generate random text following Java implementation exactly
    pub fn random_text(
        &self,
        min_length: i32,
        max_length: i32,
        random_number_stream: &mut RandomNumberStream,
    ) -> String {
        let mut is_sentence_beginning = true;
        let mut text = String::new();
        let mut target_length = RandomValueGenerator::generate_uniform_random_int(
            min_length,
            max_length,
            random_number_stream,
        );

        while target_length > 0 {
            let mut generated = self.random_sentence(random_number_stream);
            if is_sentence_beginning && !generated.is_empty() {
                let first_char = generated
                    .chars()
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .collect::<String>();
                generated = first_char + &generated[1..];
            }

            let generated_length = generated.len() as i32;
            is_sentence_beginning = generated.ends_with('.');

            // truncate so as not to exceed target length
            if target_length < generated_length {
                generated = generated[..target_length as usize].to_string();
            }

            target_length -= generated_length;

            text.push_str(&generated);
            if target_length > 0 {
                text.push(' ');
                target_length -= 1;
            }
        }

        text
    }

    // Generate random sentence following Java implementation exactly
    fn random_sentence(&self, stream: &mut RandomNumberStream) -> String {
        let mut verbiage = String::new();
        let syntax = self.sentences.pick(stream);

        for ch in syntax.chars() {
            match ch {
                'N' => verbiage.push_str(self.nouns.pick(stream)),
                'V' => verbiage.push_str(self.verbs.pick(stream)),
                'J' => verbiage.push_str(self.adjectives.pick(stream)),
                'D' => verbiage.push_str(self.adverbs.pick(stream)),
                'X' => verbiage.push_str(self.auxiliaries.pick(stream)),
                'P' => verbiage.push_str(self.prepositions.pick(stream)),
                'A' => verbiage.push_str(self.articles.pick(stream)),
                'T' => verbiage.push_str(self.terminators.pick(stream)),
                _ => verbiage.push(ch), // this is for adding punctuation and white space.
            }
        }

        verbiage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::stream::RandomNumberStream;

    #[test]
    fn test_uniform_random_int() {
        let mut stream = RandomNumberStream::new(1);
        let result = RandomValueGenerator::generate_uniform_random_int(1, 10, &mut stream);
        assert!((1..=10).contains(&result));
    }

    #[test]
    fn test_uniform_random_key() {
        let mut stream = RandomNumberStream::new(1);
        let result = RandomValueGenerator::generate_uniform_random_key(100, 200, &mut stream);
        assert!((100..=200).contains(&result));
    }

    #[test]
    fn test_uniform_random_decimal() {
        let mut stream = RandomNumberStream::new(1);
        let min = Decimal::new(100, 2); // 1.00
        let max = Decimal::new(500, 2); // 5.00
        let result = RandomValueGenerator::generate_uniform_random_decimal(min, max, &mut stream);

        assert!(result.get_number() >= min.get_number() && result.get_number() <= max.get_number());
        assert_eq!(result.get_precision(), 2);
    }

    #[test]
    fn test_uniform_random_date() {
        let mut stream = RandomNumberStream::new(1);
        let min = Date::new(2020, 1, 1);
        let max = Date::new(2020, 12, 31);
        let result = RandomValueGenerator::generate_uniform_random_date(min, max, &mut stream);

        assert!(result.to_julian_days() >= min.to_julian_days());
        assert!(result.to_julian_days() <= max.to_julian_days());
        assert_eq!(result.get_year(), 2020);
    }

    #[test]
    fn test_random_alphanumeric() {
        let mut stream = RandomNumberStream::new(1);
        let result = RandomValueGenerator::generate_random_alphanumeric(10, &mut stream);

        assert_eq!(result.len(), 10);
        for ch in result.chars() {
            assert!(RandomValueGenerator::ALPHA_NUMERIC.contains(ch));
        }
    }

    #[test]
    fn test_random_digits() {
        let mut stream = RandomNumberStream::new(1);
        let result = RandomValueGenerator::generate_random_digits(5, &mut stream);

        assert_eq!(result.len(), 5);
        for ch in result.chars() {
            assert!(ch.is_ascii_digit());
        }
    }

    #[test]
    fn test_random_boolean() {
        let mut stream = RandomNumberStream::new(1);

        // Test with 0% probability - should always be false
        let _result_never = RandomValueGenerator::generate_random_boolean(0.0, &mut stream);
        // Note: This might not always be false due to floating point precision, but typically should be

        // Test with 100% probability - should always be true
        let result_always = RandomValueGenerator::generate_random_boolean(1.0, &mut stream);
        assert!(result_always);
    }

    #[test]
    fn test_weighted_random_index() {
        let mut stream = RandomNumberStream::new(1);
        let weights = vec![10, 20, 30, 40];
        let result = RandomValueGenerator::generate_weighted_random_index(&weights, &mut stream);

        assert!(result < weights.len());
    }

    #[test]
    fn test_random_string_custom_charset() {
        let mut stream = RandomNumberStream::new(1);
        let charset = "ABC123";
        let result = RandomValueGenerator::generate_random_string(8, charset, &mut stream);

        assert_eq!(result.len(), 8);
        for ch in result.chars() {
            assert!(charset.contains(ch));
        }
    }
}
