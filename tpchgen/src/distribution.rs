use std::{
    collections::HashMap,
    io::{BufRead},
    num::ParseIntError,
};

use thiserror::Error;

use crate::rng::TpchRng;

/// TPC-H distributions seed file.
pub(crate) const DISTS_SEED: &str = include_str!("dists.dss");

#[derive(Default, Debug, Clone)]
pub struct Distribution {
    name: String,
    values: Vec<String>,
    weights: Vec<i32>,
    distribution: Vec<String>,
    max_weight: i32,
}

impl Distribution {
    /// Create a new distribution.
    pub fn new(name: &str, distribution: &HashMap<String, i32>) -> Self {
        let mut dist = Distribution {
            name: name.to_string(),
            weights: vec![0; distribution.len()],
            values: vec![],
            distribution: vec![],
            max_weight: 0,
        };

        let mut running_weight = 0;
        let mut index = 0;
        let mut is_valid_distribution = true;

        for entry in distribution {
            dist.values.push(entry.0.clone());
            running_weight += *entry.1;

            dist.weights[index] = running_weight;

            is_valid_distribution &= *entry.1 > 0;

            index += 1;
        }

        if is_valid_distribution {
            dist.max_weight = dist.weights[dist.weights.len() - 1];
            dist.distribution = vec!["".to_string(); dist.max_weight as usize];

            index = 0;

            for value in &dist.values {
                let count = distribution.get(value).unwrap();
                for _ in 0..*count {
                    dist.distribution[index] = value.clone();
                    index += 1;
                }
            }
        } else {
            dist.max_weight = -1;
            dist.distribution = vec![];
        }

        dist
    }

    fn value(&self, index: usize) -> &String {
        self.values.get(index).unwrap()
    }

    fn values(&self) -> &[String] {
        &self.values
    }

    fn weight(&self, index: usize) -> i32 {
        *self.weights.get(index).unwrap()
    }

    fn size(&self) -> usize {
        self.values.len()
    }

    fn random_value(&self, rng: &mut TpchRng) -> &str {
        let index = rng.next_int(0, self.max_weight - 1);
        self.distribution.get(index as usize).unwrap()
    }
}

#[derive(Debug, Error)]
pub enum DistributionLoaderError {
    #[error("Invalid distribution format: {0}")]
    InvalidFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseIntError),
}

pub struct DistributionLoader {}

impl DistributionLoader {
    /// Loads all distributions in the input.
    pub fn load_distributions<R: BufRead>(
        reader: R,
    ) -> Result<HashMap<String, Distribution>, DistributionLoaderError> {
        let mut distributions: HashMap<String, Distribution> = HashMap::new();

        let mut lines = reader.lines().peekable();

        while lines.peek().is_some() {
            let line = lines.next().unwrap()?;

            let parts = line
                .split_ascii_whitespace()
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>();

            if parts.len() != 2 {
                continue;
            }

            if parts.get(0).unwrap().eq_ignore_ascii_case("BEGIN") {
                let name = parts.get(1).unwrap().to_string();
                let value = Self::load_distribution(name.as_str(), &mut lines);
                let distribution = value?;
                distributions.insert(name, distribution);
            }
        }

        Ok(distributions)
    }

    /// Loads a distribution from the input by name.
    pub fn load_distribution<T: Iterator<Item = Result<String, std::io::Error>>>(
        name: &str,
        lines: T,
    ) -> Result<Distribution, DistributionLoaderError> {
        let mut count = -1 as i32;
        let mut members = HashMap::new();

        for line in lines {
            if line
                .as_ref()
                .is_ok_and(|line| Self::is_end(name, line.as_str()))
            {
                debug_assert!(
                    count as usize == members.len(),
                    "Expected {} entires in distribution {} but only {} were found",
                    count,
                    name,
                    members.len()
                );
                return Ok(Distribution::new(name, &members));
            }

            let line = line.unwrap();

            let parts = line
                .split("|")
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>();

            debug_assert!(
                parts.len() == 2,
                "Expected line to contain two parts but it contains {} parts: {}",
                parts.len(),
                line
            );

            let value = parts.get(0).unwrap().to_string();
            let weight = i32::from_str_radix(parts.get(1).unwrap(), 10)?;

            if value.eq_ignore_ascii_case("count") {
                count = weight;
            } else {
                members.insert(value, weight);
            }
        }

        Ok(Distribution::new(name, &members))
    }

    /// Checks if we reached the end of the line.
    fn is_end(name: &str, line: &str) -> bool {
        let parts = line
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();

        if parts
            .get(0)
            .is_some_and(|part| part.eq_ignore_ascii_case("END"))
        {
            debug_assert!(
                parts.len() == 2 && parts.get(1).unwrap().eq_ignore_ascii_case(name),
                "Expected end statement be 'END {}', but was '{}'",
                name,
                line
            );
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{DistributionLoader, DISTS_SEED};

    #[test]
    fn can_load_distribution_file() {
        let reader = Cursor::new(DISTS_SEED);
        let distributions = DistributionLoader::load_distributions(reader);

        assert!(distributions.is_ok());

        println!("Distributions {:#?}", distributions.unwrap());
    }
}
