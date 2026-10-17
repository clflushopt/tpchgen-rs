//! TPC-H Queries and Answers.
//!
//! This module exposes a bundled query and answer tuple that makes it
//! easier to work with them in benchmark contexts.
pub mod answers_sf1;
pub mod queries;

/// QueryAndAnswer is a struct that contains a TPC-H query and its expected answer.
pub struct QueryAndAnswer(
    &'static str, // The TPC-H query as a string
    &'static str, // The expected answer as a string
);

impl QueryAndAnswer {
    /// Creates a new QueryAndAnswer instance.
    pub fn new(num: i32, scale_factor: f64) -> Result<Self, String> {
        match (num, scale_factor) {
            (1..=22, 1.) => Ok(QueryAndAnswer(
                queries::query(num).unwrap(),
                answers_sf1::answer(num).unwrap(),
            )),
            _ => Err(format!("Invalid TPC-H query number: {} the answers are only available for queries (1 to 22) and a scale factor of 1.0", num)),
        }
    }

    /// Returns the query string.
    pub fn query(&self) -> &str {
        self.0
    }

    /// Returns the expected answer string.
    pub fn answer(&self) -> &str {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_are_well_formed_tbl() {
        for num in 1..=22 {
            let qa = QueryAndAnswer::new(num, 1.0).unwrap();
            assert!(qa.query().contains("select"), "Q{num} query malformed");
            let mut lines = qa.answer().lines().filter(|l| !l.trim().is_empty());
            let columns = lines.next().unwrap().split('|').count();
            for line in lines {
                assert_eq!(
                    line.split('|').count(),
                    columns,
                    "Q{num} row has wrong column count: {line}"
                );
            }
        }
    }

    #[test]
    fn invalid_inputs_are_rejected() {
        assert!(QueryAndAnswer::new(0, 1.0).is_err());
        assert!(QueryAndAnswer::new(23, 1.0).is_err());
        assert!(QueryAndAnswer::new(1, 10.0).is_err());
    }
}
