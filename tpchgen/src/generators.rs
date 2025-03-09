use std::vec::IntoIter;

use crate::distribution::Distribution;
use crate::distribution::Distributions;
use crate::random;
use crate::random::RandomText;
use crate::text::TextPool;

const COMMENT_AVERAGE_LENGTH: i32 = 72;

/// Generator for Nation table data
pub struct NationGenerator {
    distributions: Distributions,
    text_pool: TextPool,
}

impl NationGenerator {
    /// Creates a new NationGenerator with default distributions and text pool
    pub fn new() -> Self {
        Self::new_with_distributions_and_text_pool(
            Distributions::load_default(),
            TextPool::default(),
        )
    }

    /// Creates a NationGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        distributions: Distributions,
        text_pool: TextPool,
    ) -> Self {
        NationGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the nation rows
    pub fn iter(&self) -> NationGeneratorIterator {
        NationGeneratorIterator::new(self.distributions.nations(), &self.text_pool)
    }
}

impl IntoIterator for NationGenerator {
    type Item = Nation;
    type IntoIter = NationGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
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

/// Iterator that generates Nation rows
pub struct NationGeneratorIterator {
    nations: Distribution,
    comment_random: RandomText,
    index: usize,
}

impl NationGeneratorIterator {
    fn new(nations: &Distribution, text_pool: &TextPool) -> Self {
        NationGeneratorIterator {
            nations: nations.clone(),
            comment_random: RandomText::new(606179079, text_pool, COMMENT_AVERAGE_LENGTH as f64),
            index: 0,
        }
    }
}

impl Iterator for NationGeneratorIterator {
    type Item = Nation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.nations.size() {
            return None;
        }

        let nation = Nation {
            // n_nationkey
            n_nationkey: self.index as i64,
            // n_name
            n_name: self.nations.get_value(self.index).to_string(),
            // n_regionkey
            n_regionkey: self.nations.get_weight(self.index) as i64,
            // n_comment
            n_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(nation)
    }
}

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

/// Generator for Region table data
pub struct RegionGenerator {
    distributions: Distributions,
    text_pool: TextPool,
}

impl RegionGenerator {
    /// Creates a new RegionGenerator with default distributions and text pool
    pub fn new() -> Self {
        Self::new_with_distributions_and_text_pool(
            Distributions::load_default(),
            TextPool::default(),
        )
    }

    /// Creates a RegionGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        distributions: Distributions,
        text_pool: TextPool,
    ) -> Self {
        RegionGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the region rows
    pub fn iter(&self) -> RegionGeneratorIterator {
        RegionGeneratorIterator::new(self.distributions.regions().clone(), &self.text_pool)
    }
}

impl IntoIterator for RegionGenerator {
    type Item = Region;
    type IntoIter = RegionGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Region rows
pub struct RegionGeneratorIterator {
    regions: Distribution,
    comment_random: RandomText,
    index: usize,
}

impl RegionGeneratorIterator {
    const COMMENT_AVERAGE_LENGTH: i32 = 72;

    fn new(regions: Distribution, text_pool: &TextPool) -> Self {
        RegionGeneratorIterator {
            regions,
            comment_random: RandomText::new(
                1500869201,
                text_pool,
                Self::COMMENT_AVERAGE_LENGTH as f64,
            ),
            index: 0,
        }
    }
}

impl Iterator for RegionGeneratorIterator {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.regions.size() {
            return None;
        }

        let region = Region {
            r_regionkey: self.index as i64,
            r_name: self.regions.get_value(self.index).to_string(),
            r_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_generator() {
        let generator = NationGenerator::new();
        let nations: Vec<_> = generator.iter().collect();

        // TPC-H typically has 25 nations
        assert_eq!(nations.len(), 25);
        for nation in nations {
            println!("{:?}", nation);
        }
    }

    #[test]
    fn test_region_generator() {
        let generator = RegionGenerator::new();
        let regions: Vec<_> = generator.iter().collect();

        // TPC-H typically has 5 regions
        assert_eq!(regions.len(), 5);
        for region in regions {
            println!("{:?}", region);
        }
    }
}
