//! `tpch queries`: the TPC-H queries with substituted parameters (`qgen`).

use crate::args::{check_scale_factor, parse_scale_factor, TPCH_SCALE_FACTORS};
use crate::logging::configure_logging;
use std::io::{self, BufWriter, Write};
use tpchgen::q_and_a::qgen::{stream_order, QueryGenerator, Substitution, QUERY_COUNT};

/// Options of `tpch queries`
#[derive(clap::Args)]
#[command(next_help_heading = "Query Options")]
pub(super) struct QueriesArgs {
    /// Scale factor the queries run against (only affects Q11)
    #[arg(short, long, default_value_t = 1., value_parser = parse_scale_factor)]
    scale_factor: f64,

    /// Seed for random substitution parameters (`qgen -r`)
    ///
    /// If not specified, the queries use the validation parameters of the
    /// TPC-H specification (`qgen -d`), which produce the published answers
    /// at scale factor 1.
    #[arg(long, value_parser = parse_seed)]
    seed: Option<u32>,

    /// Generate query stream N: the queries in that stream's order (`qgen -p`)
    ///
    /// Stream 0 is the power test. If not specified, the queries are in
    /// numeric order and use stream number 0.
    #[arg(long, value_name = "N")]
    stream: Option<u32>,

    /// Which queries to generate (1 to 22) [default: all]
    ///
    /// The queries are generated in the given order, unless --stream
    /// specifies the order.
    #[arg(
        long = "queries",
        value_delimiter = ',',
        value_parser = clap::value_parser!(i32).range(1..=QUERY_COUNT as i64)
    )]
    queries: Option<Vec<i32>>,

    /// Log at info level (overrides RUST_LOG)
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Only log errors
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,
}

fn parse_seed(value: &str) -> Result<u32, String> {
    let seed: u32 = value.parse().map_err(|e| format!("{e}"))?;
    if !Substitution::is_valid_seed(seed) {
        return Err("the seed must not be a multiple of 2147483647".to_string());
    }
    Ok(seed)
}

impl QueriesArgs {
    pub(super) fn run(self) -> io::Result<()> {
        configure_logging(self.verbose, self.quiet, None);
        check_scale_factor("TPC-H", self.scale_factor, TPCH_SCALE_FACTORS);
        let substitution = match self.seed {
            Some(seed) => Substitution::Seeded(seed),
            None => Substitution::Validation,
        };
        let generator = QueryGenerator::new(self.scale_factor, substitution);
        let stream = self.stream.unwrap_or(0);
        let queries: Vec<i32> = match (self.stream, self.queries) {
            (Some(stream), selected) => {
                let mut queries = stream_order(stream).to_vec();
                if let Some(selected) = selected {
                    queries.retain(|query| selected.contains(query));
                }
                queries
            }
            // like qgen, in the given order
            (None, Some(selected)) => selected,
            (None, None) => (1..=QUERY_COUNT).collect(),
        };

        let mut out = BufWriter::new(io::stdout().lock());
        match substitution {
            Substitution::Validation => writeln!(out, "-- using default substitutions")?,
            Substitution::Seeded(seed) => writeln!(out, "-- using {seed} as a seed to the RNG")?,
        }
        for query in queries {
            let sql = generator.query(query, stream).unwrap();
            write!(out, "\n-- TPC-H Q{query}\n{sql}")?;
        }
        out.flush()
    }
}
