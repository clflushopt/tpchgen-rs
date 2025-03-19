//! Implementation of text pool and text generation.
//!
//! Most of this code has been ported from the Apache Trino TPC-H generator
//! implementation. The original code can be found in the following link:
//!
//! https://github.com/trinodb/tpch/blob/master/src/main/java/io/trino/tpch/TextPool.java

use crate::{
    distribution::{Distribution, Distributions},
    random::RowRandomInt,
};
use std::sync::OnceLock;

/// Pool of random text that follows TPC-H grammar.
#[derive(Debug, Clone)]
pub struct TextPool {
    text: Vec<u8>,
    size: i32,
}

/// The default global text pool is lazily initialized once and shared across
/// all the table generators.
static DEFAULT_TEXT_POOL: OnceLock<TextPool> = OnceLock::new();

impl TextPool {
    /// Default text pool size.
    const DEFAULT_TEXT_POOL_SIZE: i32 = 300 * 1024 * 1024;
    /// Maximum length of a sentence in the text.
    const MAX_SENTENCE_LENGTH: i32 = 256;

    /// Returns the default text pool or initializes for the first time if
    /// that's not already the case.
    pub fn get_or_init_default() -> &'static Self {
        DEFAULT_TEXT_POOL.get_or_init(|| {
            Self::new(
                Self::DEFAULT_TEXT_POOL_SIZE,
                Distributions::static_default(),
            )
        })
    }

    /// Returns a new text pool with a predefined size and set of distributions.
    pub fn new(size: i32, distributions: &Distributions) -> Self {
        let mut output = ByteArrayBuilder::new(size as usize + Self::MAX_SENTENCE_LENGTH as usize);
        let mut rng = RowRandomInt::new(933588178, i32::MAX);

        while output.length() < size as usize {
            Self::generate_sentence(distributions, &mut output, &mut rng);
        }

        output.erase(output.length() - size as usize);
        let (text_pool_bytes, text_pool_size) = output.into_inner();

        Self {
            text: text_pool_bytes,
            size: text_pool_size as i32,
        }
    }

    /// Returns the text pool size.
    pub fn size(&self) -> i32 {
        self.size
    }

    /// Returns a chunk of text from the pool
    ///
    /// Returns the text from the pool between the given begin and end indices.
    pub fn text(&self, begin: i32, end: i32) -> &str {
        // get slice of bytes (note this also does bounds checks)
        let result: &[u8] = &self.text[begin as usize..end as usize];
        // Safety: text pool contains only ASCII
        unsafe { std::str::from_utf8_unchecked(result) }
    }

    fn generate_sentence(
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.grammar().random_value(random);
        let max_length = syntax.len();

        for i in (0..max_length).step_by(2) {
            match syntax.chars().nth(i).unwrap() {
                'V' => Self::generate_verb_phrase(distributions, builder, random),
                'N' => Self::generate_noun_phrase(distributions, builder, random),
                'P' => {
                    let preposition = distributions.prepositions().random_value(random);
                    builder.append(preposition);
                    builder.append(" the ");
                    Self::generate_noun_phrase(distributions, builder, random);
                }
                'T' => {
                    builder.erase(1);
                    let terminator = distributions.terminators().random_value(random);
                    builder.append(terminator);
                }
                c => panic!("Unknown token '{}'", c),
            };

            if builder.get_last_char() != ' ' {
                builder.append(" ");
            }
        }
    }

    fn generate_verb_phrase(
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.verb_phrase().random_value(random);
        let max_length = syntax.len();

        for i in (0..max_length).step_by(2) {
            let source = match syntax.chars().nth(i).unwrap() {
                'D' => distributions.adverbs(),
                'V' => distributions.verbs(),
                'X' => distributions.auxiliaries(),
                c => panic!("Unknown token '{}'", c),
            };

            // pick a random word
            let word = source.random_value(random);
            builder.append(word);

            // add a space
            builder.append(" ");
        }
    }

    fn generate_noun_phrase(
        distributions: &Distributions,
        builder: &mut ByteArrayBuilder,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.noun_phrase().random_value(random);
        let max_length = syntax.len();

        for i in 0..max_length {
            let c = syntax.chars().nth(i).unwrap();

            let source = match c {
                'A' => distributions.articles(),
                'J' => distributions.adjectives(),
                'D' => distributions.adverbs(),
                'N' => distributions.nouns(),
                ',' => {
                    builder.erase(1);
                    builder.append(", ");
                    continue;
                }
                ' ' => continue,
                c => panic!("Unknown token '{}'", c),
            };

            // pick a random word
            let word = source.random_value(random);
            builder.append(word);
            builder.append(" ");
        }
    }
}

/// Helper struct for efficiently building the text pool
#[derive(Default)]
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

    /// Return the inner bytes and length of the builder.
    pub fn into_inner(self) -> (Vec<u8>, usize) {
        (self.bytes, self.length)
    }

    fn get_last_char(&self) -> char {
        self.bytes[self.length - 1] as char
    }
}

#[derive(Debug)]
pub struct TextPoolGenerator {
    size: usize,

    grammars: ParsedDistribution,
    noun_phrases: ParsedDistribution,
    verb_phrases: ParsedDistribution,
    prepositions: IndexedDistribution,
    terminators: IndexedDistribution,
    adverbs: IndexedDistribution,
    verbs: IndexedDistribution,
    auxiliaries: IndexedDistribution,
    articles: IndexedDistribution,
    adjectives: IndexedDistribution,
    nouns: IndexedDistribution,
}

impl TextPoolGenerator {
    const MAX_SENTENCE_LENGTH: usize = 256;

    pub fn new(size: usize, distributions: &Distributions) -> Self {
        TextPoolGenerator {
            size,
            grammars: ParsedDistribution::new(distributions.grammar()),
            noun_phrases: ParsedDistribution::new(distributions.noun_phrase()),
            verb_phrases: ParsedDistribution::new(distributions.verb_phrase()),
            prepositions: IndexedDistribution::new(distributions.prepositions()),
            terminators: IndexedDistribution::new(distributions.terminators()),
            adverbs: IndexedDistribution::new(distributions.adverbs()),
            verbs: IndexedDistribution::new(distributions.verbs()),
            auxiliaries: IndexedDistribution::new(distributions.auxiliaries()),
            articles: IndexedDistribution::new(distributions.articles()),
            adjectives: IndexedDistribution::new(distributions.adjectives()),
            nouns: IndexedDistribution::new(distributions.nouns()),
        }
    }

    pub fn generate(&mut self) -> String {
        let mut output = String::with_capacity(self.size + Self::MAX_SENTENCE_LENGTH);
        let mut random_int = RowRandomInt::new(933588178, i32::MAX);

        while output.len() < self.size {
            self.generate_sentence(&mut output, &mut random_int);
        }
        output.truncate(self.size);
        output
    }

    fn generate_sentence(&self, builder: &mut String, random: &mut RowRandomInt) {
        let index = self.grammars.get_random_index(random);
        for token in self.grammars.get_tokens(index) {
            match token {
                'V' => self.generate_verb_phrase(builder, random),
                'N' => self.generate_noun_phrase(builder, random),
                'P' => {
                    let preposition = self.prepositions.random_value(random);
                    builder.push_str(preposition);
                    builder.push_str(" the ");
                    self.generate_noun_phrase(builder, random);
                }
                'T' => {
                    // trim trailing space
                    // terminators should abut previous word
                    builder.pop();
                    let terminator = self.terminators.random_value(random);
                    builder.push_str(terminator);
                }
                _ => panic!("Unknown token '{}'", token),
            }

            if !builder.ends_with(' ') {
                builder.push(' ');
            }
        }
    }

    fn generate_verb_phrase(&self, builder: &mut String, random: &mut RowRandomInt) {
        let index = self.verb_phrases.get_random_index(random);
        for token in self.verb_phrases.get_tokens(index) {
            match token {
                'D' => builder.push_str(self.adverbs.random_value(random)),
                'V' => builder.push_str(self.verbs.random_value(random)),
                'X' => builder.push_str(self.auxiliaries.random_value(random)),
                _ => panic!("Unknown token '{}'", token),
            }

            // string may end with a comma or such
            builder.push_str(self.verb_phrases.get_bonus_text(index));

            // add a space
            builder.push(' ');
        }
    }

    fn generate_noun_phrase(&self, builder: &mut String, random: &mut RowRandomInt) {
        let index = self.noun_phrases.get_random_index(random);
        for token in self.noun_phrases.get_tokens(index) {
            match token {
                'A' => builder.push_str(self.articles.random_value(random)),
                'J' => builder.push_str(self.adjectives.random_value(random)),
                'D' => builder.push_str(self.adverbs.random_value(random)),
                'N' => builder.push_str(self.nouns.random_value(random)),
                _ => panic!("Unknown token '{}'", token),
            }

            // string may end with a comma or such
            builder.push_str(self.noun_phrases.get_bonus_text(index));

            // add a space
            builder.push(' ');
        }
    }
}

#[derive(Debug)]
struct IndexedDistribution {
    random_table: Vec<String>,
}

impl IndexedDistribution {
    fn new(distribution: &Distribution) -> Self {
        let max_weight = distribution.get_weight(distribution.size() - 1);
        let mut random_table = vec![String::new(); max_weight as usize];

        let mut value_index = 0;
        for (i, item) in random_table.iter_mut().enumerate() {
            if i >= distribution.get_weight(value_index) as usize {
                value_index += 1;
            }
            *item = distribution.get_value(value_index).to_string();
        }

        IndexedDistribution { random_table }
    }

    fn random_value(&self, random: &mut RowRandomInt) -> &str {
        let random_index = random.next_int(0, self.random_table.len() as i32 - 1) as usize;
        &self.random_table[random_index]
    }
}

#[derive(Debug)]
struct ParsedDistribution {
    parsed_distribution: Vec<Vec<char>>,
    bonus_text: Vec<String>,
    random_table: Vec<usize>,
}

impl ParsedDistribution {
    fn new(distribution: &Distribution) -> Self {
        let size = distribution.size();
        let mut parsed_distribution = Vec::with_capacity(size);
        let mut bonus_text = Vec::with_capacity(size);

        for i in 0..size {
            let value = distribution.get_value(i);
            let tokens: Vec<&str> = value.split_whitespace().collect();

            let mut chars = Vec::with_capacity(tokens.len());
            for token in &tokens {
                chars.push(token.chars().next().unwrap());
                bonus_text.push(token[1..].to_string());
            }
            parsed_distribution.push(chars);
        }

        let max_weight = distribution.get_weight(size - 1);
        let mut random_table = vec![0; max_weight as usize];

        let mut value_index = 0;
        for (i, item) in random_table.iter_mut().enumerate() {
            if i >= distribution.get_weight(value_index) as usize {
                value_index += 1;
            }
            *item = value_index;
        }

        ParsedDistribution {
            parsed_distribution,
            bonus_text,
            random_table,
        }
    }

    fn get_random_index(&self, random: &mut RowRandomInt) -> usize {
        let random_index = random.next_int(0, self.random_table.len() as i32 - 1) as usize;
        self.random_table[random_index]
    }

    fn get_tokens(&self, index: usize) -> &[char] {
        &self.parsed_distribution[index]
    }

    fn get_bonus_text(&self, index: usize) -> &str {
        &self.bonus_text[index]
    }
}
