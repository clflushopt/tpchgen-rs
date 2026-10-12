use crate::distribution::string_values_distribution::StringValuesDistribution;
use crate::random::stream::RandomNumberStream;
use std::sync::OnceLock;

static SYLLABLES_DISTRIBUTION: OnceLock<StringValuesDistribution> = OnceLock::new();

/// A single-column `.dst` word list.
macro_rules! english_distribution {
    ($name:ident, $file:literal) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name {
            inner: &'static StringValuesDistribution,
        }

        impl $name {
            pub fn new() -> Self {
                static DIST: OnceLock<StringValuesDistribution> = OnceLock::new();
                let inner = DIST.get_or_init(|| {
                    StringValuesDistribution::build_string_values_distribution($file, 1, 1)
                        .expect(concat!("Failed to load ", $file))
                });
                Self { inner }
            }

            pub fn pick(&self, stream: &mut RandomNumberStream) -> &'static str {
                self.inner.pick_random_value(0, 0, stream)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

english_distribution!(AdjectivesDistribution, "adjectives.dst");
english_distribution!(AdverbsDistribution, "adverbs.dst");
english_distribution!(ArticlesDistribution, "articles.dst");
english_distribution!(AuxiliariesDistribution, "auxiliaries.dst");
english_distribution!(PrepositionsDistribution, "prepositions.dst");
english_distribution!(NounsDistribution, "nouns.dst");
english_distribution!(SentencesDistribution, "sentences.dst");
english_distribution!(TerminatorsDistribution, "terminators.dst");
english_distribution!(VerbsDistribution, "verbs.dst");

pub fn get_syllables_distribution() -> &'static StringValuesDistribution {
    SYLLABLES_DISTRIBUTION.get_or_init(|| {
        StringValuesDistribution::build_string_values_distribution("syllables.dst", 1, 1)
            .expect("Failed to load syllables distribution")
    })
}
