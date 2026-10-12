//! Implementation of text pool and text generation.
//!
//! Most of this code has been ported from the Trino TPC-H generator
//! implementation. The original code can be found in the following link:
//!
//! <https://github.com/trinodb/tpch/blob/master/src/main/java/io/trino/tpch/TextPool.java>

use crate::{distribution::Distributions, random::RowRandomInt};
use std::sync::OnceLock;

/// Pool of random text that follows TPC-H grammar.
#[derive(Debug, Clone)]
pub struct TextPool {
    /// Bytes making up the text pool, exact size.
    text: Vec<u8>,
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
        let mut rng = RowRandomInt::new(933588178, i32::MAX);
        let mut text_bytes = Vec::with_capacity(size as usize + Self::MAX_SENTENCE_LENGTH as usize);

        while text_bytes.len() < size as usize {
            Self::generate_sentence(distributions, &mut text_bytes, &mut rng);
        }
        text_bytes.truncate(size as usize);

        Self { text: text_bytes }
    }

    /// Returns the text pool size.
    pub fn size(&self) -> i32 {
        // Cast is fine since we truncated the bytes to `size` in `new`, which
        // is an i32.
        self.text.len() as i32
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
        output: &mut Vec<u8>,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.grammar().random_value(random);
        let max_length = syntax.len();

        for c in syntax.chars().take(max_length).step_by(2) {
            match c {
                'V' => Self::generate_verb_phrase(distributions, output, random),
                'N' => Self::generate_noun_phrase(distributions, output, random),
                'P' => {
                    let preposition = distributions.prepositions().random_value(random);
                    output.extend_from_slice(preposition.as_bytes());
                    output.extend_from_slice(b" the ");
                    Self::generate_noun_phrase(distributions, output, random);
                }
                'T' => {
                    output.pop().expect("at least one byte");
                    let terminator = distributions.terminators().random_value(random);
                    output.extend_from_slice(terminator.as_bytes());
                }
                c => panic!("Unknown token '{}'", c),
            };

            let last = output.last().copied().expect("at least one byte");
            if last != b' ' {
                output.push(b' ');
            }
        }
    }

    fn generate_verb_phrase(
        distributions: &Distributions,
        output: &mut Vec<u8>,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.verb_phrase().random_value(random);
        let max_length = syntax.len();

        for c in syntax.chars().take(max_length).step_by(2) {
            let source = match c {
                'D' => distributions.adverbs(),
                'V' => distributions.verbs(),
                'X' => distributions.auxiliaries(),
                c => panic!("Unknown token '{}'", c),
            };

            // pick a random word
            let word = source.random_value(random);
            output.extend_from_slice(word.as_bytes());

            // add a space
            output.push(b' ');
        }
    }

    fn generate_noun_phrase(
        distributions: &Distributions,
        output: &mut Vec<u8>,
        random: &mut RowRandomInt,
    ) {
        let syntax = distributions.noun_phrase().random_value(random);
        let max_length = syntax.len();

        for c in syntax.chars().take(max_length) {
            let source = match c {
                'A' => distributions.articles(),
                'J' => distributions.adjectives(),
                'D' => distributions.adverbs(),
                'N' => distributions.nouns(),
                ',' => {
                    output.pop().expect("at least one byte");
                    output.extend_from_slice(b", ");
                    continue;
                }
                ' ' => continue,
                c => panic!("Unknown token '{}'", c),
            };

            // pick a random word
            let word = source.random_value(random);
            output.extend_from_slice(word.as_bytes());
            output.push(b' ');
        }
    }
}
