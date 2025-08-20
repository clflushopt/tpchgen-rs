//! TPCH data generation CLI with a dbgen compatible API.
//!
//! This crate provides a CLI for generating TPCH data and tries to remain close
//! API wise to the original dbgen tool, as in we use the same command line flags
//! and arguments.
//!
//! ```
//! USAGE:
//!     tpchgen-cli [OPTIONS]
//!
//! OPTIONS:
//!     -h, --help                    Prints help information
//!     -V, --version                 Prints version information
//!     -s, --scale-factor <FACTOR>  Scale factor for the data generation (default: 1)
//!     -T, --tables <TABLES>        Comma-separated list of tables to generate (default: all)
//!     -f, --format <FORMAT>        Output format: tbl, csv, or parquet (default: tbl)
//!     -o, --output-dir <DIR>       Output directory (default: current directory)
//!     -p, --parts <N>              Number of parts to split generation into (default: 1)
//!         --part <N>               Which part to generate (1-based, default: 1)
//!     -n, --num-threads <N>        Number of threads to use (default: number of CPUs)
//!     -c, --parquet-compression <C> Parquet compression codec, e.g., SNAPPY, ZSTD(1), UNCOMPRESSED (default: SNAPPY)
//!         --parquet-row-group-size <N> Number of rows per row group in Parquet files (default: 1048576)
//!     -v, --verbose                Verbose output
//!         --stdout                 Write output to stdout instead of files
//!```
//!
//! # Logging:
//! Use the `-v` flag or `RUST_LOG` environment variable to control logging output.
//!
//! `-v` sets the log level to `info` and ignores the `RUST_LOG` environment variable.
//!
//! # Examples
//! ```
//! # see all info output
//! tpchgen-cli -s 1 -v
//!
//! # same thing using RUST_LOG
//! RUST_LOG=info tpchgen-cli -s 1
//!
//! # see all debug output
//! RUST_LOG=debug tpchgen -s 1
//! ```
mod csv;
mod generate;
mod parquet;
mod plan;
mod runner;
mod statistics;
mod tbl;

use crate::generate::Sink;
use crate::parquet::*;
use crate::plan::{
    GenerationPlan, OutputLocation, OutputPartitionPlan, DEFAULT_PARQUET_ROW_GROUP_BYTES,
};
use crate::statistics::WriteStatistics;
use ::parquet::basic::Compression;
use clap::builder::TypedValueParser;
use clap::{Parser, ValueEnum};
use log::{debug, info, LevelFilter};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufWriter, Stdout, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use tpchgen::distribution::Distributions;
use tpchgen::text::TextPool;

#[derive(Parser)]
#[command(name = "tpchgen")]
#[command(version)]
#[command(about = "TPC-H Data Generator", long_about = None)]
struct Cli {
    /// Scale factor to create
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short = 'T', long = "tables", value_delimiter = ',', value_parser = TableValueParser)]
    tables: Option<Vec<Table>>,

    /// Number of part(itions) to generate (manual parallel generation)
    #[arg(short, long)]
    parts: Option<i32>,

    /// Which part(ition) to generate (1-based)
    ///
    /// If not specified, generates all parts
    #[arg(long)]
    part: Option<i32>,

    /// Output format: tbl, csv, parquet
    #[arg(short, long, default_value = "tbl")]
    format: OutputFormat,

    /// The number of threads for parallel generation, defaults to the number of CPUs
    #[arg(short, long, default_value_t = num_cpus::get())]
    num_threads: usize,

    /// Parquet block compression format.
    ///
    /// Supported values: UNCOMPRESSED, ZSTD(N), SNAPPY, GZIP, LZO, BROTLI, LZ4
    ///
    /// Note to use zstd you must supply the "compression" level (1-22)
    /// as a number in parentheses, e.g. `ZSTD(1)` for level 1 compression.
    ///
    /// Using `ZSTD` results in the best compression, but is about 2x slower than
    /// UNCOMPRESSED. For example, for the lineitem table at SF=10
    ///
    ///   ZSTD(1):      1.9G  (0.52 GB/sec)
    ///   SNAPPY:       2.4G  (0.75 GB/sec)
    ///   UNCOMPRESSED: 3.8G  (1.41 GB/sec)
    #[arg(short = 'c', long, default_value = "SNAPPY")]
    parquet_compression: Compression,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Write the output to stdout instead of a file.
    #[arg(long, default_value_t = false)]
    stdout: bool,

    /// Target size in row group bytes in Parquet files
    ///
    /// Row groups are the typical unit of parallel processing and compression
    /// in Parquet. With many query engines, smaller row groups enable better
    /// parallelism and lower peak memory use but may reduce compression
    /// efficiency.
    ///
    /// Note: parquet files are limited to 32k row groups, so at high scale
    /// factors, the row group size may be increased to keep the number of row
    /// groups under this limit.
    ///
    /// Typical values range from 10MB to 100MB.
    #[arg(long, default_value_t = DEFAULT_PARQUET_ROW_GROUP_BYTES)]
    parquet_row_group_bytes: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Table {
    Nation,
    Region,
    Part,
    Supplier,
    Partsupp,
    Customer,
    Orders,
    Lineitem,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone)]
struct TableValueParser;

impl TypedValueParser for TableValueParser {
    type Value = Table;

    /// Parse the value into a Table enum.
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = value
            .to_str()
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd))?;
        Table::from_str(value)
            .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd))
    }

    fn possible_values(
        &self,
    ) -> Option<Box<dyn Iterator<Item = clap::builder::PossibleValue> + '_>> {
        Some(Box::new(
            [
                clap::builder::PossibleValue::new("region").help("Region table (alias: r)"),
                clap::builder::PossibleValue::new("nation").help("Nation table (alias: n)"),
                clap::builder::PossibleValue::new("supplier").help("Supplier table (alias: s)"),
                clap::builder::PossibleValue::new("customer").help("Customer table (alias: c)"),
                clap::builder::PossibleValue::new("part").help("Part table (alias: P)"),
                clap::builder::PossibleValue::new("partsupp").help("PartSupp table (alias: S)"),
                clap::builder::PossibleValue::new("orders").help("Orders table (alias: O)"),
                clap::builder::PossibleValue::new("lineitem").help("LineItem table (alias: L)"),
            ]
            .into_iter(),
        ))
    }
}

impl FromStr for Table {
    type Err = &'static str;

    /// Returns the table enum value from the given string full name or abbreviation
    ///
    /// The original dbgen tool allows some abbreviations to mean two different tables
    /// like 'p' which aliases to both 'part' and 'partsupp'. This implementation does
    /// not support this since it just adds unnecessary complexity and confusion so we
    /// only support the exclusive abbreviations.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" | "nation" => Ok(Table::Nation),
            "r" | "region" => Ok(Table::Region),
            "s" | "supplier" => Ok(Table::Supplier),
            "P" | "part" => Ok(Table::Part),
            "S" | "partsupp" => Ok(Table::Partsupp),
            "c" | "customer" => Ok(Table::Customer),
            "O" | "orders" => Ok(Table::Orders),
            "L" | "lineitem" => Ok(Table::Lineitem),
            _ => Err("Invalid table name {s}"),
        }
    }
}

impl Table {
    fn name(&self) -> &'static str {
        match self {
            Table::Nation => "nation",
            Table::Region => "region",
            Table::Part => "part",
            Table::Supplier => "supplier",
            Table::Partsupp => "partsupp",
            Table::Customer => "customer",
            Table::Orders => "orders",
            Table::Lineitem => "lineitem",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Tbl,
    Csv,
    Parquet,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    cli.main().await
}

impl Cli {
    /// Main function to run the generation
    async fn main(self) -> io::Result<()> {
        if self.verbose {
            // explicitly set logging to info / stdout
            env_logger::builder().filter_level(LevelFilter::Info).init();
            info!("Verbose output enabled (ignoring RUST_LOG environment variable)");
        } else {
            env_logger::init();
            debug!("Logging configured from environment variables");
        }

        // Create output directory if it doesn't exist and we are not writing to stdout.
        if !self.stdout {
            fs::create_dir_all(&self.output_dir)?;
        }

        // Determine which tables to generate
        let tables: Vec<Table> = if let Some(tables) = self.tables.as_ref() {
            tables.clone()
        } else {
            vec![
                Table::Nation,
                Table::Region,
                Table::Part,
                Table::Supplier,
                Table::Partsupp,
                Table::Customer,
                Table::Orders,
                Table::Lineitem,
            ]
        };

        // Warn if parquet specific options are set but not generating parquet
        if self.format != OutputFormat::Parquet {
            if self.parquet_compression != Compression::SNAPPY {
                eprintln!(
                    "Warning: Parquet compression option set but not generating Parquet files"
                );
            }
            if self.parquet_row_group_bytes != DEFAULT_PARQUET_ROW_GROUP_BYTES {
                eprintln!(
                    "Warning: Parquet row group size option set but not generating Parquet files"
                );
            }
        }

        // Determine what files to generate
        let mut output_partition_plans = vec![];
        for table in &tables {
            self.create_plans(*table, &mut output_partition_plans)?;
        }

        // force the creation of the distributions and text pool to so it doesn't
        // get charged to the first table
        let start = Instant::now();
        debug!("Creating distributions and text pool");
        Distributions::static_default();
        TextPool::get_or_init_default();
        let elapsed = start.elapsed();
        info!("Created static distributions and text pools in {elapsed:?}");

        // Run
        let runner = runner::PlanRunner::new(output_partition_plans, self.num_threads);
        runner.run().await?;
        info!("Generation complete!");
        Ok(())
    }

    /// Appends  OutputPartitionPlan(s) for the given table to `plans`
    fn create_plans(
        &self,
        table: Table,
        plans: &mut Vec<OutputPartitionPlan>,
    ) -> Result<(), io::Error> {
        let generation_plan = GenerationPlan::try_new(
            table,
            self.format,
            self.scale_factor,
            self.part,
            self.parts,
            self.parquet_row_group_bytes,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let output_location = if self.stdout {
            OutputLocation::Stdout
        } else {
            // todo add partition names here (.1, .2, etc)
            let file_name = self.output_filename(table);
            OutputLocation::File(self.output_dir.join(file_name))
        };

        let plan = OutputPartitionPlan::new(
            table,
            self.scale_factor,
            self.format,
            self.parquet_compression,
            output_location,
            generation_plan,
        );

        plans.push(plan);
        Ok(())
    }

    /// return the output filename for the given table
    fn output_filename(&self, table: Table) -> String {
        let extension = match self.format {
            OutputFormat::Tbl => "tbl",
            OutputFormat::Csv => "csv",
            OutputFormat::Parquet => "parquet",
        };
        format!("{}.{extension}", table.name())
    }
}

impl IntoSize for BufWriter<Stdout> {
    fn into_size(self) -> Result<usize, io::Error> {
        // we can't get the size of stdout, so just return 0
        Ok(0)
    }
}

impl IntoSize for BufWriter<File> {
    fn into_size(self) -> Result<usize, io::Error> {
        let file = self.into_inner()?;
        let metadata = file.metadata()?;
        Ok(metadata.len() as usize)
    }
}

/// Wrapper around a buffer writer that counts the number of buffers and bytes written
struct WriterSink<W: Write> {
    statistics: WriteStatistics,
    inner: W,
}

impl<W: Write> WriterSink<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            statistics: WriteStatistics::new("buffers"),
        }
    }
}

impl<W: Write + Send> Sink for WriterSink<W> {
    fn sink(&mut self, buffer: &[u8]) -> Result<(), io::Error> {
        self.statistics.increment_chunks(1);
        self.statistics.increment_bytes(buffer.len());
        self.inner.write_all(buffer)
    }

    fn flush(mut self) -> Result<(), io::Error> {
        self.inner.flush()
    }
}
