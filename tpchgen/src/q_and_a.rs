use crate::{answers, queries};

/// TPC-H Queries and Answers.
///
/// This module exposes a bundled query and answer tuple that makes it
/// easier to work with them in benchmark contexts.

/// QueryAndAnswer is a struct that contains a TPC-H query and its expected answer.
pub struct QueryAndAnswer(
    &'static str, // The TPC-H query as a string
    &'static str, // The expected answer as a string
);

impl QueryAndAnswer {
    /// Creates a new QueryAndAnswer instance.
    pub fn new(num: i32) -> Self {
        match num {
            1..=22 => Self(queries::query(num).unwrap(), answers::answer(num).unwrap()),
            _ => unreachable!("Invalid TPC-H query number: {}", num),
        }
    }

    /// Returns the query string.
    pub fn query(&self) -> &str {
        &self.0
    }

    /// Returns the expected answer string.
    pub fn answer(&self) -> &str {
        &self.1
    }
}
