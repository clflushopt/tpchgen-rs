use std::{collections::HashMap, io::BufRead, num::ParseIntError, sync::{Arc, Once}};

use thiserror::Error;

use crate::{rng::TpchRng, textgen::RandomInt};

/// TPC-H distributions seed file.
pub(crate) const DISTS_SEED: &str = include_str!("dists.dss");


/// A mapping of values to weights for random selection
#[derive(Debug, Clone)]
pub struct Distribution {
    name: String,
    values: Vec<String>,
    weights: Vec<i32>,
    distribution: Option<Vec<String>>,
    max_weight: i32,
}

impl Distribution {
    pub fn new(name: String, distribution_map: HashMap<String, i32>) -> Self {
        let mut values = Vec::new();
        let mut weights = Vec::new();

        let mut running_weight = 0;
        let mut is_valid_distribution = true;

        for (key, weight) in distribution_map {
            values.push(key);
            running_weight += weight;
            weights.push(running_weight);

            is_valid_distribution &= weight > 0;
        }

        // Create distribution array for valid distributions
        let distribution = if is_valid_distribution && !values.is_empty() {
            let max_weight = weights[weights.len() - 1];
            let mut dist = vec![String::new(); max_weight as usize];

            let mut value_index = 0;
            for i in 0..max_weight {
                if i >= weights[value_index] {
                    value_index += 1;
                }
                dist[i as usize] = values[value_index].clone();
            }

            Some(dist)
        } else {
            None
        };

        let max_weight = if weights.is_empty() {
            0
        } else {
            weights[weights.len() - 1]
        };

        Self {
            name,
            values,
            weights,
            distribution,
            max_weight,
        }
    }

    /// Get the value at the specified index
    pub fn get_value(&self, index: usize) -> &str {
        &self.values[index]
    }

    /// Get the weight at the specified index
    pub fn get_weight(&self, index: usize) -> i32 {
        self.weights[index]
    }

    /// Get the size of the distribution
    pub fn size(&self) -> usize {
        self.values.len()
    }

    /// Get a random value from the distribution
    pub fn random_value(&self, random: &mut RandomInt) -> &str {
        match &self.distribution {
            Some(dist) => {
                let random_value = random.next_int(0, self.max_weight - 1) as usize;
                &dist[random_value]
            }
            None => panic!("{} does not have a distribution", self.name),
        }
    }
}

/// Container for all the distributions used in TPC-H
#[derive(Debug)]
pub struct Distributions {
    grammars: Distribution,
    noun_phrase: Distribution,
    verb_phrase: Distribution,
    prepositions: Distribution,
    nouns: Distribution,
    verbs: Distribution,
    articles: Distribution,
    adjectives: Distribution,
    adverbs: Distribution,
    auxiliaries: Distribution,
    terminators: Distribution,
    order_priorities: Distribution,
    ship_instructions: Distribution,
    ship_modes: Distribution,
    return_flags: Distribution,
    part_containers: Distribution,
    part_colors: Distribution,
    part_types: Distribution,
    market_segments: Distribution,
    nations: Distribution,
    regions: Distribution,
}

impl Distributions {
    /// Create a new Distributions instance from a map of named distributions
    pub fn new(distributions: HashMap<String, Distribution>) -> Self {
        // Helper to get distribution or panic if not found
        let get_dist = |name: &str| -> Distribution {
            distributions
                .get(name)
                .cloned()
                .unwrap_or_else(|| panic!("Distribution {} does not exist", name))
        };

        Self {
            grammars: get_dist("grammar"),
            noun_phrase: get_dist("np"),
            verb_phrase: get_dist("vp"),
            prepositions: get_dist("prepositions"),
            nouns: get_dist("nouns"),
            verbs: get_dist("verbs"),
            articles: get_dist("articles"),
            adjectives: get_dist("adjectives"),
            adverbs: get_dist("adverbs"),
            auxiliaries: get_dist("auxillaries"),
            terminators: get_dist("terminators"),
            order_priorities: get_dist("o_oprio"),
            ship_instructions: get_dist("instruct"),
            ship_modes: get_dist("smode"),
            return_flags: get_dist("rflag"),
            part_containers: get_dist("p_cntr"),
            part_colors: get_dist("colors"),
            part_types: get_dist("p_types"),
            market_segments: get_dist("msegmnt"),
            nations: get_dist("nations"),
            regions: get_dist("regions"),
        }
    }

    // Getter methods for each distribution
    pub fn get_grammars(&self) -> &Distribution {
        &self.grammars
    }

    pub fn get_noun_phrase(&self) -> &Distribution {
        &self.noun_phrase
    }

    pub fn get_verb_phrase(&self) -> &Distribution {
        &self.verb_phrase
    }

    pub fn get_prepositions(&self) -> &Distribution {
        &self.prepositions
    }

    pub fn get_nouns(&self) -> &Distribution {
        &self.nouns
    }

    pub fn get_verbs(&self) -> &Distribution {
        &self.verbs
    }

    pub fn get_articles(&self) -> &Distribution {
        &self.articles
    }

    pub fn get_adjectives(&self) -> &Distribution {
        &self.adjectives
    }

    pub fn get_adverbs(&self) -> &Distribution {
        &self.adverbs
    }

    pub fn get_auxiliaries(&self) -> &Distribution {
        &self.auxiliaries
    }

    pub fn get_terminators(&self) -> &Distribution {
        &self.terminators
    }

    pub fn get_order_priorities(&self) -> &Distribution {
        &self.order_priorities
    }

    pub fn get_ship_instructions(&self) -> &Distribution {
        &self.ship_instructions
    }

    pub fn get_ship_modes(&self) -> &Distribution {
        &self.ship_modes
    }

    pub fn get_return_flags(&self) -> &Distribution {
        &self.return_flags
    }

    pub fn get_part_containers(&self) -> &Distribution {
        &self.part_containers
    }

    pub fn get_part_colors(&self) -> &Distribution {
        &self.part_colors
    }

    pub fn get_part_types(&self) -> &Distribution {
        &self.part_types
    }

    pub fn get_market_segments(&self) -> &Distribution {
        &self.market_segments
    }

    pub fn get_nations(&self) -> &Distribution {
        &self.nations
    }

    pub fn get_regions(&self) -> &Distribution {
        &self.regions
    }

    /// Load the default distributions from the embedded resource
    pub fn get_default_distributions() -> Arc<Self> {
        static INSTANCE: Once = Once::new();
        static mut DEFAULT_DISTRIBUTIONS: Option<Arc<Distributions>> = None;

        INSTANCE.call_once(|| {
            // Load distributions from embedded dists.dss
            let distributions = load_distributions(DISTS_SEED);

            unsafe {
                DEFAULT_DISTRIBUTIONS = Some(Arc::new(distributions));
            }
        });

        unsafe { DEFAULT_DISTRIBUTIONS.as_ref().unwrap().clone() }
    }
}
/// Represents a parsed distribution for use in text generation
pub struct ParsedDistribution {
    /// Tokens for each distribution entry
    parsed_distribution: Vec<Vec<char>>,
    /// Bonus text for each distribution entry
    bonus_text: Vec<String>,
    /// Random selection table
    random_table: Vec<usize>,
}

impl ParsedDistribution {
    /// Create a new ParsedDistribution from a Distribution
    pub fn new(distribution: &Distribution) -> Self {
        let size = distribution.size();
        let mut parsed_distribution = Vec::with_capacity(size);
        let mut bonus_text = Vec::with_capacity(size);

        for i in 0..size {
            let value = distribution.get_value(i);
            let tokens: Vec<&str> = value.split_whitespace().collect();

            let mut tokens_chars = Vec::with_capacity(tokens.len());
            let mut bonuses = Vec::with_capacity(tokens.len());

            for token in tokens {
                tokens_chars.push(token.chars().next().unwrap());
                if token.len() > 1 {
                    bonuses.push(&token[1..]);
                } else {
                    bonuses.push("");
                }
            }

            parsed_distribution.push(tokens_chars);
            bonus_text.push(bonuses.join(""));
        }

        // Create random table
        let max_weight = distribution.get_weight(size - 1);
        let mut random_table = Vec::with_capacity(max_weight as usize);

        let mut value_index = 0;
        for i in 0..max_weight {
            if i >= distribution.get_weight(value_index) {
                value_index += 1;
            }
            random_table.push(value_index);
        }

        Self {
            parsed_distribution,
            bonus_text,
            random_table,
        }
    }

    /// Get a random index into the distribution
    pub fn get_random_index(&self, random: &mut RandomInt) -> usize {
        let random_index = random.next_int(0, (self.random_table.len() - 1) as i32) as usize;
        self.random_table[random_index]
    }

    /// Get the tokens for a given index
    pub fn get_tokens(&self, index: usize) -> &[char] {
        &self.parsed_distribution[index]
    }

    /// Get the bonus text for a given index
    pub fn get_bonus_text(&self, index: usize) -> &str {
        &self.bonus_text[index]
    }
}

/// Load distributions from a string containing dists.dss content
fn load_distributions(content: &str) -> Distributions {
    let mut distributions = HashMap::new();
    let mut current_name = None;
    let mut current_members = HashMap::new();
    let mut count = -1;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for begin/end markers
        if line.to_uppercase().starts_with("BEGIN") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                current_name = Some(parts[1].to_string());
                current_members = HashMap::new();
                count = -1;
            }
            continue;
        }

        if let Some(name) = &current_name {
            if line.to_uppercase().starts_with("END") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 && parts[1] == name {
                    // Finish the current distribution
                    if count >= 0 && count as usize == current_members.len() {
                        distributions.insert(
                            name.to_lowercase(),
                            Distribution::new(name.clone(), current_members.clone()),
                        );
                    } else {
                        eprintln!(
                            "Warning: Expected {} entries in distribution {}, but found {}",
                            count,
                            name,
                            current_members.len()
                        );
                    }

                    current_name = None;
                }
                continue;
            }

            // Parse a distribution line
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let value = parts[0];
                let weight: i32 = parts[1].parse().unwrap_or(0);

                if value.eq_ignore_ascii_case("count") {
                    count = weight;
                } else {
                    current_members.insert(value.to_string(), weight);
                }
            }
        }
    }

    Distributions::new(distributions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_distributions() {
        let distributions = crate::distribution::load_distributions(DISTS_SEED);
        println!("{:#?}", distributions);
    }
}