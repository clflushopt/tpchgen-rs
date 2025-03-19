use crate::random::RowRandomInt;
use std::{
    io::{self, BufRead},
    sync::LazyLock,
};

use indexmap::IndexMap;

/// TPC-H distributions seed file.
pub(crate) const DISTS_SEED: &str = include_str!("dists.dss");

/// Distribution represents a weighted collection of string values from the TPC-H specification.
/// It provides methods to access values by index or randomly based on their weights.
#[derive(Debug, Clone)]
pub struct Distribution {
    name: String,
    values: Vec<String>,
    weights: Vec<i32>,
    distribution: Option<Vec<String>>,
    max_weight: i32,
}

impl Distribution {
    /// Creates a new Distribution with the given name and weighted values.
    pub fn new(name: String, distribution: IndexMap<String, i32>) -> Self {
        let mut values = Vec::new();
        let mut weights = vec![0; distribution.len()];

        let mut running_weight = 0;
        let mut is_valid_distribution = true;

        // Process each value and its weight
        for (index, (value, weight)) in distribution.iter().enumerate() {
            values.push(value.clone());

            running_weight += weight;
            weights[index] = running_weight;

            // A valid distribution requires all weights to be positive
            is_valid_distribution &= *weight > 0;
        }

        // Only create the full distribution array for valid distributions
        // "nations" is a special case that's not a valid distribution
        let (distribution_array, max_weight) = if is_valid_distribution {
            let max = weights[weights.len() - 1];
            let mut dist = vec![String::new(); max as usize];

            let mut index = 0;
            for value in values.iter() {
                let count = distribution.get(value).unwrap();

                for _ in 0..*count {
                    dist[index] = value.clone();
                    index += 1;
                }
            }

            (Some(dist), max)
        } else {
            (None, -1)
        };

        Distribution {
            name,
            values,
            weights,
            distribution: distribution_array,
            max_weight,
        }
    }

    /// Returns the distribution name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets a value at the specified index.
    pub fn get_value(&self, index: usize) -> &str {
        &self.values[index]
    }

    /// Gets all values in this distribution.
    pub fn get_values(&self) -> &[String] {
        &self.values
    }

    /// Gets the cumulative weight at the specified index.
    pub fn get_weight(&self, index: usize) -> i32 {
        self.weights[index]
    }

    /// Gets the number of distinct values in this distribution.
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Gets a random value from this distribution using the provided random number.
    pub fn random_value(&self, random: &mut RowRandomInt) -> &str {
        if let Some(dist) = &self.distribution {
            let random_value = random.next_int(0, self.max_weight - 1);
            return &dist[random_value as usize];
        }
        unreachable!("Cannot get random value from an invalid distribution")
    }
}

/// DistributionLoader provides functionality to load TPC-H distributions from a text format.
pub struct DistributionLoader;

impl DistributionLoader {
    /// Loads distributions from a stream of lines.
    ///
    /// The format is expected to follow the TPC-H specification format where:
    /// - Lines starting with "#" are comments
    /// - Distributions start with "BEGIN <name>"
    /// - Distribution entries are formatted as "value|weight"
    /// - Distributions end with "END"
    pub fn load_distributions<I>(lines: I) -> io::Result<IndexMap<String, Distribution>>
    where
        I: Iterator<Item = io::Result<String>>,
    {
        let filtered_lines = lines.filter_map(|line_result| {
            line_result.ok().and_then(|line| {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            })
        });

        Self::load_distributions_from_filtered_lines(filtered_lines)
    }

    /// Internal method to load distributions from pre-filtered lines.
    fn load_distributions_from_filtered_lines<I>(
        lines: I,
    ) -> io::Result<IndexMap<String, Distribution>>
    where
        I: Iterator<Item = String>,
    {
        let mut distributions = IndexMap::new();
        let mut lines_iter = lines.peekable();

        while let Some(line) = lines_iter.next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("BEGIN") {
                let name = parts[1].to_string();
                let distribution = Self::load_distribution(&mut lines_iter, &name)?;
                distributions.insert(name, distribution);
            }
        }

        Ok(distributions)
    }

    /// Loads a single distribution until its END marker.
    fn load_distribution<I>(
        lines: &mut std::iter::Peekable<I>,
        name: &str,
    ) -> io::Result<Distribution>
    where
        I: Iterator<Item = String>,
    {
        let mut members = IndexMap::new();
        let mut _count = -1;

        for line in lines.by_ref() {
            if Self::is_end(&line) {
                return Ok(Distribution::new(name.to_string(), members));
            }

            let parts: Vec<&str> = line.split("|").collect::<Vec<_>>();
            if parts.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid distribution line format: {}", line),
                ));
            }

            let value = parts[0].to_string();
            let weight = match parts[1].trim().parse::<i32>() {
                Ok(w) => w,
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid distribution {}: invalid weight on line {}",
                            name, line
                        ),
                    ));
                }
            };

            if value.eq_ignore_ascii_case("count") {
                _count = weight;
            } else {
                members.insert(value, weight);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid distribution {}: no end statement", name),
        ))
    }

    /// Checks if a line is an END marker.
    fn is_end(line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        parts.first().is_some_and(|p| p.eq_ignore_ascii_case("END"))
    }
}

/// Static global instance of the default distributions.
///
/// Initialized once on first access.
static DEFAULT_DISTRIBUTIONS: LazyLock<Distributions> = LazyLock::new(Distributions::default);

/// Distributions wraps all TPC-H distributions and provides methods to access them.
#[derive(Debug, Clone)]
pub struct Distributions {
    distributions: IndexMap<String, Distribution>,
}

impl Default for Distributions {
    /// Loads the default distributions from `DISTS_SEED`.
    fn default() -> Self {
        let cursor = io::Cursor::new(DISTS_SEED);
        let lines = cursor.lines();
        let distributions = DistributionLoader::load_distributions(lines).unwrap();
        Distributions::new(distributions)
    }
}

impl Distributions {
    /// Creates a new distributions wrapper.
    pub fn new(distributions: IndexMap<String, Distribution>) -> Self {
        Distributions { distributions }
    }

    /// Returns a static reference to the default distributions.
    pub fn static_default() -> &'static Distributions {
        &DEFAULT_DISTRIBUTIONS
    }

    /// Returns the `adjectives` distribution.
    pub fn adjectives(&self) -> &Distribution {
        self.get("adjectives")
    }

    /// Returns the `adverbs` distribution.
    pub fn adverbs(&self) -> &Distribution {
        self.get("adverbs")
    }

    /// Returns the `articles` distribution.
    pub fn articles(&self) -> &Distribution {
        self.get("articles")
    }

    /// Returns the `auxillaries` distribution.
    ///
    /// P.S: The correct spelling is `auxiliaries` which is what we use.
    pub fn auxiliaries(&self) -> &Distribution {
        self.get("auxillaries")
    }

    /// Returns the `grammar` distribution.
    pub fn grammar(&self) -> &Distribution {
        self.get("grammar")
    }

    /// Returns the `category` distribution.
    pub fn category(&self) -> &Distribution {
        self.get("category")
    }

    /// Returns the `msegmnt` distribution.
    pub fn market_segments(&self) -> &Distribution {
        self.get("msegmnt")
    }

    /// Returns the `nations` distribution.
    pub fn nations(&self) -> &Distribution {
        self.get("nations")
    }

    /// Returns the `noun_phrases` distribution.
    pub fn noun_phrase(&self) -> &Distribution {
        self.get("np")
    }

    /// Returns the `nouns` distribution.
    pub fn nouns(&self) -> &Distribution {
        self.get("nouns")
    }

    /// Returns the `orders_priority` distribution.
    pub fn order_priority(&self) -> &Distribution {
        self.get("o_oprio")
    }

    /// Returns the `part_colors` distribution.
    pub fn part_colors(&self) -> &Distribution {
        self.get("colors")
    }

    /// Returns the `part_containers` distribution.
    pub fn part_containers(&self) -> &Distribution {
        self.get("p_cntr")
    }

    /// Returns the `part_types` distribution.
    pub fn part_types(&self) -> &Distribution {
        self.get("p_types")
    }

    /// Returns the `prepositions` distribution.
    pub fn prepositions(&self) -> &Distribution {
        self.get("prepositions")
    }

    /// Returns the `regions` distribution.
    pub fn regions(&self) -> &Distribution {
        self.get("regions")
    }

    /// Returns the `return_flags` distribution.
    pub fn return_flags(&self) -> &Distribution {
        self.get("rflag")
    }

    /// Returns the `ship_instructions` distribution.
    pub fn ship_instructions(&self) -> &Distribution {
        self.get("instruct")
    }

    /// Returns the `ship_modes` distribution.
    pub fn ship_modes(&self) -> &Distribution {
        self.get("smode")
    }

    /// Returns the `terminators` distribution.
    pub fn terminators(&self) -> &Distribution {
        self.get("terminators")
    }

    // Returns the `verb_phrases` distribution.
    pub fn verb_phrase(&self) -> &Distribution {
        self.get("vp")
    }

    /// Returns the `verbs` distribution.
    pub fn verbs(&self) -> &Distribution {
        self.get("verbs")
    }

    /// Returns the distribution with the specified name.
    ///
    /// # Panics
    ///  Panics if the distribution does not exist.
    pub fn get(&self, name: &str) -> &Distribution {
        self.distributions
            .get(name)
            .unwrap_or_else(|| panic!("Distribution not found: {}", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_load_empty() {
        let input = "";
        let cursor = Cursor::new(input);
        let lines = cursor.lines();
        let distributions = DistributionLoader::load_distributions(lines).unwrap();
        assert!(distributions.is_empty());
    }

    #[test]
    fn test_load_simple_distribution() {
        let input = "
            # Comment line
            BEGIN test
            value1|10
            value2|20
            END
        ";
        let cursor = Cursor::new(input);
        let lines = cursor.lines();
        let distributions = DistributionLoader::load_distributions(lines).unwrap();

        assert_eq!(distributions.len(), 1);
        assert!(distributions.contains_key("test"));

        let test_dist = distributions.get("test").unwrap();
        assert_eq!(test_dist.size(), 2);
        assert_eq!(test_dist.get_value(0), "value1");
        assert_eq!(test_dist.get_value(1), "value2");
        assert_eq!(test_dist.get_weight(0), 10);
        assert_eq!(test_dist.get_weight(1), 30); // Cumulative weight
    }

    #[test]
    fn test_load_multiple_distributions() {
        let input = "
            BEGIN first
            a|5
            b|10
            END

            BEGIN second
            x|2
            y|3
            z|4
            END
        ";
        let cursor = Cursor::new(input);
        let lines = cursor.lines();
        let distributions = DistributionLoader::load_distributions(lines).unwrap();

        assert_eq!(distributions.len(), 2);
        assert!(distributions.contains_key("first"));
        assert!(distributions.contains_key("second"));

        let first_dist = distributions.get("first").unwrap();
        assert_eq!(first_dist.size(), 2);

        let second_dist = distributions.get("second").unwrap();
        assert_eq!(second_dist.size(), 3);
    }

    #[test]
    fn test_error_on_invalid_weight() {
        let input = "
            BEGIN test
            value|invalid
            END
        ";
        let cursor = Cursor::new(input);
        let lines = cursor.lines();
        let result = DistributionLoader::load_distributions(lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_on_missing_end() {
        let input = "
            BEGIN test
            value|10
        ";
        let cursor = Cursor::new(input);
        let lines = cursor.lines();
        let result = DistributionLoader::load_distributions(lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_with_default_seeds_file() {
        let expected_distributions = vec![
            "category",
            "p_cntr",
            "instruct",
            "msegmnt",
            "p_names",
            "nations",
            "nations2",
            "regions",
            "o_oprio",
            "regions",
            "rflag",
            "smode",
            "p_types",
            "colors",
            "articles",
            "nouns",
            "verbs",
            "adjectives",
            "adverbs",
            "auxillaries",
            "prepositions",
            "terminators",
            "grammar",
            "np",
            "vp",
            "Q13a",
            "Q13b",
        ];

        let cursor = Cursor::new(DISTS_SEED);
        let lines = cursor.lines();
        let distributions = DistributionLoader::load_distributions(lines).unwrap();
        assert_eq!(distributions.len(), 26);

        for name in expected_distributions {
            assert!(
                distributions.contains_key(name),
                "missing distribution: {}",
                name
            );
        }
    }
}
