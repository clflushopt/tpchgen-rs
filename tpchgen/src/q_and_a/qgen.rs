//! TPC-H query generation with parameter substitution, a port of `qgen`.
//!
//! The TPC-H queries are templates with substitution parameters (`:1`, `:2`,
//! ...) that are replaced with values chosen as specified in clause 2.4 of the
//! TPC-H specification. [`QueryGenerator`] generates executable queries the
//! same way `qgen` from the TPC-H tools does, either with the validation
//! parameters ([`Substitution::Validation`], `qgen -d`) or with random
//! parameters ([`Substitution::Seeded`], `qgen -r <seed>`).
//!
//! The parameters match those of `qgen` version 2.17 and later. Earlier
//! versions (such as 2.14) chose the Q4 date from the wrong range and produced
//! invalid Q22 country codes.
//!
//! The generated SQL is the query template with the `qgen` directives
//! resolved for standard SQL: the row count limits of Q2, Q3, Q10, Q18 and
//! Q21 become a `limit` clause, and the other directives (`:x`, `:o`, ...) are
//! removed. The only change to the templates themselves is that Q1 subtracts
//! `interval ':1' day` instead of `interval ':1' day (3)`: the interval
//! precision is standard SQL but not supported by common engines such as
//! DataFusion and DuckDB, and the specification permits this change (clause
//! 2.2.3.3).
//!
//! # Example
//! ```
//! # use tpchgen::q_and_a::qgen::{QueryGenerator, Substitution};
//! let generator = QueryGenerator::new(1.0, Substitution::Validation);
//! assert_eq!(generator.parameters(3).unwrap(), ["BUILDING", "1995-03-15"]);
//!
//! let sql = generator.query(3, 0).unwrap();
//! assert!(sql.contains("c_mktsegment = 'BUILDING'"));
//! assert!(sql.ends_with("\to_orderdate\nlimit 10;\n"));
//!
//! // Random parameters, as `qgen -r 101`
//! let generator = QueryGenerator::new(1.0, Substitution::Seeded(101));
//! assert_eq!(generator.parameters(1).unwrap(), ["77"]);
//! ```

use crate::distribution::{Distribution, Distributions};
use crate::random::RowRandomInt;

/// Number of queries in a query stream
pub const QUERY_COUNT: i32 = 22;

/// The query templates, as distributed with `qgen`
const TEMPLATES: [&str; QUERY_COUNT as usize] = [
    include_str!("templates/1.sql"),
    include_str!("templates/2.sql"),
    include_str!("templates/3.sql"),
    include_str!("templates/4.sql"),
    include_str!("templates/5.sql"),
    include_str!("templates/6.sql"),
    include_str!("templates/7.sql"),
    include_str!("templates/8.sql"),
    include_str!("templates/9.sql"),
    include_str!("templates/10.sql"),
    include_str!("templates/11.sql"),
    include_str!("templates/12.sql"),
    include_str!("templates/13.sql"),
    include_str!("templates/14.sql"),
    include_str!("templates/15.sql"),
    include_str!("templates/16.sql"),
    include_str!("templates/17.sql"),
    include_str!("templates/18.sql"),
    include_str!("templates/19.sql"),
    include_str!("templates/20.sql"),
    include_str!("templates/21.sql"),
    include_str!("templates/22.sql"),
];

/// The validation parameters of each query (clause 2.4.x.3 of the
/// specification, `defaults` in `varsub.c`).
///
/// The Q11 fraction depends on the scale factor and is computed separately.
const VALIDATION_PARAMETERS: [&[&str]; QUERY_COUNT as usize] = [
    &["90"],
    &["15", "BRASS", "EUROPE"],
    &["BUILDING", "1995-03-15"],
    &["1993-07-01"],
    &["ASIA", "1994-01-01"],
    &["1994-01-01", ".06", "24"],
    &["FRANCE", "GERMANY"],
    &["BRAZIL", "AMERICA", "ECONOMY ANODIZED STEEL"],
    &["green"],
    &["1993-10-01"],
    &["GERMANY", "0.0001"],
    &["MAIL", "SHIP", "1994-01-01"],
    &["special", "requests"],
    &["1995-09-01"],
    &["1996-01-01"],
    &[
        "Brand#45",
        "MEDIUM POLISHED",
        "49",
        "14",
        "23",
        "45",
        "19",
        "3",
        "36",
        "9",
    ],
    &["Brand#23", "MED BOX"],
    &["300"],
    &["Brand#12", "Brand#23", "Brand#34", "1", "10", "20"],
    &["forest", "1994-01-01", "CANADA"],
    &["SAUDI ARABIA"],
    &["13", "31", "23", "29", "30", "18", "17"],
];

/// The order of the queries in each query stream (`permutation` in
/// `permute.h`, Appendix A of the specification).
///
/// Stream `s` uses row `s % 41`.
const STREAM_ORDER: [[i32; QUERY_COUNT as usize]; 41] = [
    [
        14, 2, 9, 20, 6, 17, 18, 8, 21, 13, 3, 22, 16, 4, 11, 15, 1, 10, 19, 5, 7, 12,
    ],
    [
        21, 3, 18, 5, 11, 7, 6, 20, 17, 12, 16, 15, 13, 10, 2, 8, 14, 19, 9, 22, 1, 4,
    ],
    [
        6, 17, 14, 16, 19, 10, 9, 2, 15, 8, 5, 22, 12, 7, 13, 18, 1, 4, 20, 3, 11, 21,
    ],
    [
        8, 5, 4, 6, 17, 7, 1, 18, 22, 14, 9, 10, 15, 11, 20, 2, 21, 19, 13, 16, 12, 3,
    ],
    [
        5, 21, 14, 19, 15, 17, 12, 6, 4, 9, 8, 16, 11, 2, 10, 18, 1, 13, 7, 22, 3, 20,
    ],
    [
        21, 15, 4, 6, 7, 16, 19, 18, 14, 22, 11, 13, 3, 1, 2, 5, 8, 20, 12, 17, 10, 9,
    ],
    [
        10, 3, 15, 13, 6, 8, 9, 7, 4, 11, 22, 18, 12, 1, 5, 16, 2, 14, 19, 20, 17, 21,
    ],
    [
        18, 8, 20, 21, 2, 4, 22, 17, 1, 11, 9, 19, 3, 13, 5, 7, 10, 16, 6, 14, 15, 12,
    ],
    [
        19, 1, 15, 17, 5, 8, 9, 12, 14, 7, 4, 3, 20, 16, 6, 22, 10, 13, 2, 21, 18, 11,
    ],
    [
        8, 13, 2, 20, 17, 3, 6, 21, 18, 11, 19, 10, 15, 4, 22, 1, 7, 12, 9, 14, 5, 16,
    ],
    [
        6, 15, 18, 17, 12, 1, 7, 2, 22, 13, 21, 10, 14, 9, 3, 16, 20, 19, 11, 4, 8, 5,
    ],
    [
        15, 14, 18, 17, 10, 20, 16, 11, 1, 8, 4, 22, 5, 12, 3, 9, 21, 2, 13, 6, 19, 7,
    ],
    [
        1, 7, 16, 17, 18, 22, 12, 6, 8, 9, 11, 4, 2, 5, 20, 21, 13, 10, 19, 3, 14, 15,
    ],
    [
        21, 17, 7, 3, 1, 10, 12, 22, 9, 16, 6, 11, 2, 4, 5, 14, 8, 20, 13, 18, 15, 19,
    ],
    [
        2, 9, 5, 4, 18, 1, 20, 15, 16, 17, 7, 21, 13, 14, 19, 8, 22, 11, 10, 3, 12, 6,
    ],
    [
        16, 9, 17, 8, 14, 11, 10, 12, 6, 21, 7, 3, 15, 5, 22, 20, 1, 13, 19, 2, 4, 18,
    ],
    [
        1, 3, 6, 5, 2, 16, 14, 22, 17, 20, 4, 9, 10, 11, 15, 8, 12, 19, 18, 13, 7, 21,
    ],
    [
        3, 16, 5, 11, 21, 9, 2, 15, 10, 18, 17, 7, 8, 19, 14, 13, 1, 4, 22, 20, 6, 12,
    ],
    [
        14, 4, 13, 5, 21, 11, 8, 6, 3, 17, 2, 20, 1, 19, 10, 9, 12, 18, 15, 7, 22, 16,
    ],
    [
        4, 12, 22, 14, 5, 15, 16, 2, 8, 10, 17, 9, 21, 7, 3, 6, 13, 18, 11, 20, 19, 1,
    ],
    [
        16, 15, 14, 13, 4, 22, 18, 19, 7, 1, 12, 17, 5, 10, 20, 3, 9, 21, 11, 2, 6, 8,
    ],
    [
        20, 14, 21, 12, 15, 17, 4, 19, 13, 10, 11, 1, 16, 5, 18, 7, 8, 22, 9, 6, 3, 2,
    ],
    [
        16, 14, 13, 2, 21, 10, 11, 4, 1, 22, 18, 12, 19, 5, 7, 8, 6, 3, 15, 20, 9, 17,
    ],
    [
        18, 15, 9, 14, 12, 2, 8, 11, 22, 21, 16, 1, 6, 17, 5, 10, 19, 4, 20, 13, 3, 7,
    ],
    [
        7, 3, 10, 14, 13, 21, 18, 6, 20, 4, 9, 8, 22, 15, 2, 1, 5, 12, 19, 17, 11, 16,
    ],
    [
        18, 1, 13, 7, 16, 10, 14, 2, 19, 5, 21, 11, 22, 15, 8, 17, 20, 3, 4, 12, 6, 9,
    ],
    [
        13, 2, 22, 5, 11, 21, 20, 14, 7, 10, 4, 9, 19, 18, 6, 3, 1, 8, 15, 12, 17, 16,
    ],
    [
        14, 17, 21, 8, 2, 9, 6, 4, 5, 13, 22, 7, 15, 3, 1, 18, 16, 11, 10, 12, 20, 19,
    ],
    [
        10, 22, 1, 12, 13, 18, 21, 20, 2, 14, 16, 7, 15, 3, 4, 17, 5, 19, 6, 8, 9, 11,
    ],
    [
        10, 8, 9, 18, 12, 6, 1, 5, 20, 11, 17, 22, 16, 3, 13, 2, 15, 21, 14, 19, 7, 4,
    ],
    [
        7, 17, 22, 5, 3, 10, 13, 18, 9, 1, 14, 15, 21, 19, 16, 12, 8, 6, 11, 20, 4, 2,
    ],
    [
        2, 9, 21, 3, 4, 7, 1, 11, 16, 5, 20, 19, 18, 8, 17, 13, 10, 12, 15, 6, 14, 22,
    ],
    [
        15, 12, 8, 4, 22, 13, 16, 17, 18, 3, 7, 5, 6, 1, 9, 11, 21, 10, 14, 20, 19, 2,
    ],
    [
        15, 16, 2, 11, 17, 7, 5, 14, 20, 4, 21, 3, 10, 9, 12, 8, 13, 6, 18, 19, 22, 1,
    ],
    [
        1, 13, 11, 3, 4, 21, 6, 14, 15, 22, 18, 9, 7, 5, 10, 20, 12, 16, 17, 8, 19, 2,
    ],
    [
        14, 17, 22, 20, 8, 16, 5, 10, 1, 13, 2, 21, 12, 9, 4, 18, 3, 7, 6, 19, 15, 11,
    ],
    [
        9, 17, 7, 4, 5, 13, 21, 18, 11, 3, 22, 1, 6, 16, 20, 14, 15, 10, 8, 2, 12, 19,
    ],
    [
        13, 14, 5, 22, 19, 11, 9, 6, 18, 15, 8, 10, 7, 4, 17, 16, 3, 1, 12, 2, 21, 20,
    ],
    [
        20, 5, 4, 14, 11, 1, 6, 16, 8, 22, 7, 3, 2, 12, 21, 19, 17, 13, 10, 15, 18, 9,
    ],
    [
        3, 7, 14, 15, 6, 5, 21, 20, 18, 10, 4, 16, 19, 1, 13, 9, 8, 17, 11, 12, 22, 2,
    ],
    [
        13, 15, 17, 1, 22, 11, 3, 4, 7, 20, 14, 21, 9, 8, 2, 18, 16, 6, 10, 12, 5, 19,
    ],
];

/// Returns the order of the queries in query stream `stream`, as `qgen -p
/// <stream>`.
///
/// Stream 0 is the power test, streams 1 and up are the throughput test
/// streams.
pub fn stream_order(stream: u32) -> [i32; QUERY_COUNT as usize] {
    STREAM_ORDER[stream as usize % STREAM_ORDER.len()]
}

/// How the substitution parameters of the queries are chosen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substitution {
    /// The validation parameters of the TPC-H specification (`qgen -d`).
    ///
    /// At scale factor 1 the query results must match the answers in
    /// [`answers_sf1`](super::answers_sf1).
    Validation,
    /// Random parameters from a random number generator with the given seed
    /// (`qgen -r <seed>`).
    ///
    /// The parameters of a query only depend on the seed, not on the query
    /// stream. The specification requires a different seed for each query
    /// stream, conventionally the seed of the power test plus the stream
    /// number.
    Seeded(u32),
}

impl Substitution {
    /// Returns true if `seed` can be used for [`Substitution::Seeded`].
    ///
    /// Seeds that are a multiple of 2,147,483,647 (such as 0) make every
    /// random number 0 (`qgen` hangs for them).
    pub fn is_valid_seed(seed: u32) -> bool {
        seed as i64 % RowRandomInt::MODULUS != 0
    }
}

/// Generates TPC-H queries with substituted parameters, like `qgen`.
///
/// See the [module documentation](self) for details.
#[derive(Debug, Clone)]
pub struct QueryGenerator<'a> {
    scale_factor: f64,
    substitution: Substitution,
    distributions: &'a Distributions,
}

impl QueryGenerator<'static> {
    /// Creates a query generator for the given scale factor (`qgen -s`)
    ///
    /// # Panics
    ///
    /// Panics for seeds that are not [valid](Substitution::is_valid_seed).
    pub fn new(scale_factor: f64, substitution: Substitution) -> Self {
        Self::new_with_distributions(scale_factor, substitution, Distributions::static_default())
    }
}

impl<'a> QueryGenerator<'a> {
    /// Creates a query generator with the specified distributions
    pub fn new_with_distributions(
        scale_factor: f64,
        substitution: Substitution,
        distributions: &'a Distributions,
    ) -> Self {
        if let Substitution::Seeded(seed) = substitution {
            assert!(
                Substitution::is_valid_seed(seed),
                "invalid qgen seed {seed}: seeds must not be a multiple of {}",
                RowRandomInt::MODULUS
            );
        }
        QueryGenerator {
            scale_factor,
            substitution,
            distributions,
        }
    }

    /// Returns the substitution parameters (`:1`, `:2`, ...) of query `query`
    /// (1 to 22), or `None` for other query numbers.
    pub fn parameters(&self, query: i32) -> Option<Vec<String>> {
        if !(1..=QUERY_COUNT).contains(&query) {
            return None;
        }
        let mut parameters = match self.substitution {
            Substitution::Validation => VALIDATION_PARAMETERS[query as usize - 1]
                .iter()
                .map(|parameter| parameter.to_string())
                .collect(),
            Substitution::Seeded(seed) => {
                ParameterRandom::new(seed, query, self.distributions).parameters(query)
            }
        };
        if query == 11 {
            // The Q11 fraction is scaled in both modes
            parameters[1] = format!("{:11.10}", Self::Q11_FRACTION / self.scale_factor);
        }
        Some(parameters)
    }

    /// Returns query `query` (1 to 22) as SQL with its parameters substituted,
    /// or `None` for other query numbers.
    ///
    /// `stream` is the number of the query stream the query is part of. It
    /// only affects the name of the view Q15 creates (`revenue<stream>`).
    ///
    /// Q15 consists of three statements, which create a view, query it and
    /// drop it again. All other queries are a single statement.
    pub fn query(&self, query: i32, stream: u32) -> Option<String> {
        let parameters = self.parameters(query)?;
        Some(substitute(
            TEMPLATES[query as usize - 1],
            query,
            &parameters,
            stream,
        ))
    }

    /// Returns the 22 queries of query stream `stream` in the order the
    /// stream executes them (see [`stream_order`]), as `(query number, SQL)`
    /// pairs.
    pub fn stream(&self, stream: u32) -> Vec<(i32, String)> {
        stream_order(stream)
            .into_iter()
            .map(|query| (query, self.query(query, stream).unwrap()))
            .collect()
    }

    /// Fraction of the total value of all available parts in Q11 at scale
    /// factor 1 (`Q11_FRACTION`)
    const Q11_FRACTION: f64 = 0.0001;
}

/// Replaces the substitution parameters and directives of a `qgen` template
/// (`qsub` in `qgen.c`).
fn substitute(template: &str, query: i32, parameters: &[String], stream: u32) -> String {
    let mut sql = String::new();
    let mut limit = -1;
    for line in template.split_inclusive('\n') {
        let line = strip_comment(line);
        let directive = line.trim();
        if let Some(rows) = directive
            .strip_prefix(":n")
            .or_else(|| directive.strip_prefix(":N"))
        {
            limit = atoi(rows);
            continue;
        }
        // directives without output in standard SQL (explain, output file,
        // begin and end transaction, database)
        if matches!(
            directive,
            ":x" | ":X" | ":o" | ":O" | ":b" | ":B" | ":e" | ":E" | ":c" | ":C"
        ) {
            continue;
        }

        let mut rest = line;
        while let Some(position) = rest.find(':') {
            sql.push_str(&rest[..position]);
            rest = &rest[position + 1..];
            let tag = rest.chars().next().unwrap_or('\0');
            if tag.is_ascii_digit() {
                let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
                let index: usize = rest[..digits].parse().unwrap();
                sql.push_str(&parameters[index - 1]);
                rest = &rest[digits..];
            } else {
                match tag {
                    's' | 'S' => sql.push_str(&stream.to_string()),
                    'q' | 'Q' => sql.push_str(&query.to_string()),
                    _ => panic!("unexpected directive ':{tag}' in Q{query} template"),
                }
                rest = &rest[1..];
            }
        }
        sql.push_str(rest);
    }

    if limit > 0 {
        let end = sql
            .trim_end()
            .strip_suffix(';')
            .expect("query ends with a semicolon")
            .len();
        sql.truncate(end);
        sql.push_str(&format!("\nlimit {limit};\n"));
    }
    sql
}

/// Removes a `--` comment from a template line.
///
/// Like `qgen`, this only recognizes a comment that starts at the first `-`
/// of the line, so it never mistakes a subtraction or a date for a comment.
fn strip_comment(line: &str) -> &str {
    match line.find('-') {
        Some(position) if line[position + 1..].starts_with('-') => &line[..position],
        _ => line,
    }
}

/// Parses a leading integer like C's `atoi`
fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (sign, digits) = match s.strip_prefix('-') {
        Some(digits) => (-1, digits),
        None => (1, s.strip_prefix('+').unwrap_or(s)),
    };
    let length = digits.bytes().take_while(u8::is_ascii_digit).count();
    sign * digits[..length].parse::<i32>().unwrap_or(0)
}

/// The random number stream of one query in `qgen` (`varsub.c`)
struct ParameterRandom<'a> {
    random: RowRandomInt,
    distributions: &'a Distributions,
}

impl<'a> ParameterRandom<'a> {
    /// Part sizes, permuted for Q16
    const SIZES: std::ops::RangeInclusive<i64> = 1..=50;
    /// Country codes minus 10, permuted for Q22
    const COUNTRY_CODES: std::ops::RangeInclusive<i64> = 0..=24;

    /// Creates the random number stream of query `query`.
    ///
    /// `qgen` seeds stream 0 with the seed, and then each query's stream with
    /// the next random value of stream 0.
    fn new(seed: u32, query: i32, distributions: &'a Distributions) -> Self {
        let mut stream_zero = RowRandomInt::new(seed as i64, 1);
        let mut query_seed = seed as i64;
        for _ in 0..query {
            query_seed = stream_zero.next_rand();
        }
        ParameterRandom {
            random: RowRandomInt::new(query_seed, 1),
            distributions,
        }
    }

    /// A uniformly distributed integer (`UnifInt` and `RANDOM`)
    fn int(&mut self, low: i32, high: i32) -> i32 {
        self.random.next_int(low, high)
    }

    /// A random value of a distribution and its index (`pick_str`)
    fn pick(&mut self, distribution: &'a Distribution) -> (usize, &'a str) {
        let max_weight = distribution.get_weight(distribution.size() - 1);
        let weight = self.int(1, max_weight);
        let index = (0..distribution.size())
            .find(|&index| distribution.get_weight(index) >= weight)
            .unwrap();
        (index, distribution.get_value(index))
    }

    /// A random permutation of the given values (`permute`)
    fn permute(&mut self, values: impl Iterator<Item = i64>) -> Vec<i64> {
        let mut values: Vec<_> = values.collect();
        let count = values.len() as i32;
        for i in 0..count {
            let source = self.int(i, count - 1);
            values.swap(source as usize, i as usize);
        }
        values
    }

    /// A random month between `first` and `last` months after January 1993,
    /// formatted as the date of its first day
    fn month(&mut self, first: i32, last: i32) -> String {
        let month = self.int(first, last);
        format!("19{:02}-{:02}-01", 93 + month / 12, month % 12 + 1)
    }

    /// The first day of a random year between 1993 and 1997
    fn year(&mut self) -> String {
        format!("19{:02}-01-01", self.int(93, 97))
    }

    /// A random brand (`Brand#MN`)
    fn brand(&mut self) -> String {
        let manufacturer = self.int(1, 5);
        let brand = self.int(1, 5);
        format!("Brand#{manufacturer}{brand}")
    }

    /// Two different random values of a distribution
    fn pick_two(&mut self, distribution: &'a Distribution) -> [String; 2] {
        let (first, first_value) = self.pick(distribution);
        loop {
            let (second, second_value) = self.pick(distribution);
            if second != first {
                return [first_value.to_string(), second_value.to_string()];
            }
        }
    }

    /// The random parameters of query `query` (`varsub` in `varsub.c`)
    fn parameters(&mut self, query: i32) -> Vec<String> {
        let d = self.distributions;
        match query {
            1 => vec![self.int(60, 120).to_string()],
            2 => {
                let size = self.int(1, 50).to_string();
                let (_, part_type) = self.pick(d.part_types());
                // the last word of the type, e.g. "BRASS"
                let material = part_type.rsplit(' ').next().unwrap();
                let (_, region) = self.pick(d.regions());
                vec![size, material.to_string(), region.to_string()]
            }
            3 => {
                let (_, segment) = self.pick(d.market_segments());
                let day = self.int(1, 31);
                vec![segment.to_string(), format!("1995-03-{day:02}")]
            }
            4 => vec![self.month(0, 57)],
            5 => {
                let (_, region) = self.pick(d.regions());
                vec![region.to_string(), self.year()]
            }
            6 => {
                let date = self.year();
                let discount = format!("0.{:02}", self.int(2, 9));
                let quantity = self.int(24, 25).to_string();
                vec![date, discount, quantity]
            }
            7 => self.pick_two(d.nations2()).to_vec(),
            8 => {
                let (nation, nation_name) = self.pick(d.nations2());
                // the nations distribution encodes the region key in its
                // cumulative weights
                let region = d.nations().get_weight(nation) as usize;
                let (_, part_type) = self.pick(d.part_types());
                vec![
                    nation_name.to_string(),
                    d.regions().get_value(region).to_string(),
                    part_type.to_string(),
                ]
            }
            9 => vec![self.pick(d.part_colors()).1.to_string()],
            10 => vec![self.month(1, 24)],
            11 => {
                let (_, nation) = self.pick(d.nations2());
                // the fraction is set by the caller, it depends on the scale
                vec![nation.to_string(), String::new()]
            }
            12 => {
                let [first, second] = self.pick_two(d.ship_modes());
                vec![first, second, self.year()]
            }
            13 => {
                let (_, first) = self.pick(d.q13a());
                let (_, second) = self.pick(d.q13b());
                vec![first.to_string(), second.to_string()]
            }
            14 => vec![self.month(0, 59)],
            15 => vec![self.month(0, 57)],
            16 => {
                let brand = self.brand();
                let (_, part_type) = self.pick(d.part_types());
                // the type without its last word, e.g. "MEDIUM POLISHED"
                let (prefix, _) = part_type.rsplit_once(' ').unwrap();
                let sizes = self.permute(Self::SIZES);
                let mut parameters = vec![brand, prefix.to_string()];
                parameters.extend(sizes[..8].iter().map(i64::to_string));
                parameters
            }
            17 => {
                let brand = self.brand();
                let (_, container) = self.pick(d.part_containers());
                vec![brand, container.to_string()]
            }
            18 => vec![self.int(312, 315).to_string()],
            19 => vec![
                self.brand(),
                self.brand(),
                self.brand(),
                self.int(1, 10).to_string(),
                self.int(10, 20).to_string(),
                self.int(20, 30).to_string(),
            ],
            20 => {
                let (_, color) = self.pick(d.part_colors());
                let date = self.year();
                let (_, nation) = self.pick(d.nations2());
                vec![color.to_string(), date, nation.to_string()]
            }
            21 => vec![self.pick(d.nations2()).1.to_string()],
            22 => self.permute(Self::COUNTRY_CODES)[..7]
                .iter()
                .map(|code| (10 + code).to_string())
                .collect(),
            _ => unreachable!("invalid query number {query}"),
        }
    }
}
