//! TPC-H Data Generator Library
//!
//! This crate provides both a command-line tool and a library for generating
//! TPC-H benchmark data in various formats (TBL, CSV, Parquet).
//!
//! # Examples
//!
//! ```no_run
//! use tpchgen_cli::{TpchGenerator, Table, OutputFormat};
//! use std::path::PathBuf;
//!
//! # async fn example() -> std::io::Result<()> {
//! let generator = TpchGenerator::builder()
//!     .with_scale_factor(10.0)
//!     .with_output_dir(PathBuf::from("./data"))
//!     .with_tables(vec![Table::Customer, Table::Orders])
//!     .with_format(OutputFormat::Parquet)
//!     .with_num_threads(8)
//!     .build();
//!
//! generator.generate().await?;
//! # Ok(())
//! # }
//! ```

pub use crate::plan::{GenerationPlan, DEFAULT_PARQUET_ROW_GROUP_BYTES};
pub use ::parquet::basic::Compression;

pub mod csv;
pub mod generate;
pub mod output_plan;
pub mod parquet;
pub mod plan;
pub mod runner;
pub mod statistics;
pub mod tbl;

use crate::generate::Sink;
use crate::parquet::IntoSize;
use crate::statistics::WriteStatistics;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufWriter, Stdout, Write};
use std::str::FromStr;

/// Wrapper around a buffer writer that counts the number of buffers and bytes written
pub struct WriterSink<W: Write> {
    statistics: WriteStatistics,
    inner: W,
}

impl<W: Write> WriterSink<W> {
    pub fn new(inner: W) -> Self {
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

/// TPC-H table types
///
/// Represents the 8 tables in the TPC-H benchmark schema.
/// Tables are ordered by size (smallest to largest at SF=1).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Table {
    /// Nation table (25 rows)
    Nation,
    /// Region table (5 rows)
    Region,
    /// Part table (200,000 rows at SF=1)
    Part,
    /// Supplier table (10,000 rows at SF=1)
    Supplier,
    /// Part-Supplier relationship table (800,000 rows at SF=1)
    Partsupp,
    /// Customer table (150,000 rows at SF=1)
    Customer,
    /// Orders table (1,500,000 rows at SF=1)
    Orders,
    /// Line item table (6,000,000 rows at SF=1)
    Lineitem,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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

/// Output format for generated data
///
/// # Format Details
///
/// - **TBL**: Pipe-delimited format compatible with original dbgen tool
/// - **CSV**: Comma-separated values with proper escaping
/// - **Parquet**: Columnar Apache Parquet format with configurable compression
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputFormat {
    /// TBL format (pipe-delimited, dbgen-compatible)
    Tbl,
    /// CSV format (comma-separated values)
    Csv,
    /// Apache Parquet format (columnar, compressed)
    Parquet,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tbl" => Ok(OutputFormat::Tbl),
            "csv" => Ok(OutputFormat::Csv),
            "parquet" => Ok(OutputFormat::Parquet),
            _ => Err(format!(
                "Invalid output format: {s}. Valid formats are: tbl, csv, parquet"
            )),
        }
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Tbl => write!(f, "tbl"),
            OutputFormat::Csv => write!(f, "csv"),
            OutputFormat::Parquet => write!(f, "parquet"),
        }
    }
}

/// Configuration for TPC-H data generation
///
/// This struct holds all the parameters needed to generate TPC-H benchmark data.
/// It's typically not constructed directly - use [`TpchGeneratorBuilder`] instead.
///
/// # Examples
///
/// ```no_run
/// use tpchgen_cli::{GeneratorConfig, OutputFormat};
///
/// // Usually you would use TpchGenerator::builder() instead
/// let config = GeneratorConfig {
///     scale_factor: 10.0,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Scale factor (e.g., 1.0 for 1GB, 10.0 for 10GB)
    pub scale_factor: f64,
    /// Output directory for generated files
    pub output_dir: std::path::PathBuf,
    /// Tables to generate (if None, generates all tables)
    pub tables: Option<Vec<Table>>,
    /// Output format (TBL, CSV, or Parquet)
    pub format: OutputFormat,
    /// Number of threads for parallel generation
    pub num_threads: usize,
    /// Parquet compression format
    pub parquet_compression: Compression,
    /// Target row group size in bytes for Parquet files
    pub parquet_row_group_bytes: i64,
    /// Number of partitions to generate (if None, generates a single file per table)
    pub parts: Option<i32>,
    /// Specific partition to generate (1-based, requires parts to be set)
    pub part: Option<i32>,
    /// Write output to stdout instead of files
    pub stdout: bool,
    /// CSV delimiter character (only applies to CSV format)
    pub csv_delimiter: char,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            output_dir: std::path::PathBuf::from("."),
            tables: None,
            format: OutputFormat::Tbl,
            num_threads: num_cpus::get(),
            parquet_compression: Compression::SNAPPY,
            parquet_row_group_bytes: DEFAULT_PARQUET_ROW_GROUP_BYTES,
            parts: None,
            part: None,
            stdout: false,
            csv_delimiter: ',',
        }
    }
}

/// TPC-H data generator
///
/// The main entry point for generating TPC-H benchmark data.
/// Use the builder pattern via [`TpchGenerator::builder()`] to configure and create instances.
///
/// # Examples
///
/// ```no_run
/// use tpchgen_cli::{TpchGenerator, Table, OutputFormat};
/// use std::path::PathBuf;
/// use ::parquet::basic::ZstdLevel;
/// # async fn example() -> std::io::Result<()> {
/// // Generate all tables at scale factor 1 in TBL format
/// TpchGenerator::builder()
///     .with_scale_factor(1.0)
///     .with_output_dir(PathBuf::from("./data"))
///     .build()
///     .generate()
///     .await?;
///
/// // Generate specific tables in Parquet format with compression
/// TpchGenerator::builder()
///     .with_scale_factor(10.0)
///     .with_output_dir(PathBuf::from("./benchmark_data"))
///     .with_tables(vec![Table::Orders, Table::Lineitem])
///     .with_format(OutputFormat::Parquet)
///     .with_parquet_compression(tpchgen_cli::Compression::ZSTD(ZstdLevel::try_new(1).unwrap()))
///     .with_num_threads(16)
///     .build()
///     .generate()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct TpchGenerator {
    config: GeneratorConfig,
}

impl TpchGenerator {
    /// Create a new builder for configuring the generator
    ///
    /// This is the recommended way to construct a [`TpchGenerator`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tpchgen_cli::TpchGenerator;
    ///
    /// let generator = TpchGenerator::builder()
    ///     .with_scale_factor(1.0)
    ///     .build();
    /// ```
    pub fn builder() -> TpchGeneratorBuilder {
        TpchGeneratorBuilder::new()
    }

    /// Generate TPC-H data with the configured settings
    ///
    /// This async method performs the actual data generation, creating files
    /// in the configured output directory (or writing to stdout if configured).
    ///
    /// # Returns
    ///
    /// - `Ok(())` on successful generation
    /// - `Err(io::Error)` if file I/O or generation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tpchgen_cli::TpchGenerator;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// TpchGenerator::builder()
    ///     .with_scale_factor(1.0)
    ///     .build()
    ///     .generate()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate(self) -> io::Result<()> {
        use crate::output_plan::OutputPlanGenerator;
        use crate::runner::PlanRunner;
        use log::info;
        use std::time::Instant;
        use tpchgen::distribution::Distributions;
        use tpchgen::text::TextPool;

        let config = self.config;

        // Create output directory if it doesn't exist and we are not writing to stdout
        if !config.stdout {
            std::fs::create_dir_all(&config.output_dir)?;
        }

        // Determine which tables to generate
        let tables: Vec<Table> = if let Some(tables) = config.tables {
            tables
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

        // Determine what files to generate
        let mut output_plan_generator = OutputPlanGenerator::new(
            config.format,
            config.scale_factor,
            config.parquet_compression,
            config.parquet_row_group_bytes,
            config.stdout,
            config.output_dir,
            config.csv_delimiter,
        );

        for table in tables {
            output_plan_generator.generate_plans(table, config.part, config.parts)?;
        }
        let output_plans = output_plan_generator.build();

        // Force the creation of the distributions and text pool so it doesn't
        // get charged to the first table
        let start = Instant::now();
        Distributions::static_default();
        TextPool::get_or_init_default();
        let elapsed = start.elapsed();
        info!("Created static distributions and text pools in {elapsed:?}");

        // Run
        let runner = PlanRunner::new(output_plans, config.num_threads);
        runner.run().await?;
        info!("Generation complete!");
        Ok(())
    }
}

/// Builder for constructing a [`TpchGenerator`]
///
/// Provides a fluent interface for configuring TPC-H data generation parameters.
/// All builder methods can be chained, and calling [`build()`](TpchGeneratorBuilder::build)
/// produces a [`TpchGenerator`] ready to generate data.
///
/// # Defaults
///
/// - Scale factor: 1.0
/// - Output directory: current directory (".")
/// - Tables: all 8 tables
/// - Format: TBL
/// - Threads: number of CPUs
/// - Parquet compression: SNAPPY
/// - Row group size: 7MB
///
/// # Examples
///
/// ```no_run
/// use tpchgen_cli::{TpchGenerator, Table, OutputFormat, Compression};
/// use std::path::PathBuf;
/// use ::parquet::basic::ZstdLevel;
///
/// # async fn example() -> std::io::Result<()> {
/// let generator = TpchGenerator::builder()
///     .with_scale_factor(100.0)
///     .with_output_dir(PathBuf::from("/data/tpch"))
///     .with_tables(vec![Table::Lineitem, Table::Orders])
///     .with_format(OutputFormat::Parquet)
///     .with_parquet_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap()))
///     .with_num_threads(32)
///     .build();
///
/// generator.generate().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TpchGeneratorBuilder {
    config: GeneratorConfig,
}

impl TpchGeneratorBuilder {
    /// Create a new builder with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use tpchgen_cli::TpchGeneratorBuilder;
    ///
    /// let builder = TpchGeneratorBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
        }
    }

    /// Returns the scale factor.
    pub fn scale_factor(&self) -> f64 {
        self.config.scale_factor
    }

    /// Set the scale factor (e.g., 1.0 for 1GB, 10.0 for 10GB)
    pub fn with_scale_factor(mut self, scale_factor: f64) -> Self {
        self.config.scale_factor = scale_factor;
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, output_dir: impl Into<std::path::PathBuf>) -> Self {
        self.config.output_dir = output_dir.into();
        self
    }

    /// Set which tables to generate (default: all tables)
    pub fn with_tables(mut self, tables: Vec<Table>) -> Self {
        self.config.tables = Some(tables);
        self
    }

    /// Set the output format (default: TBL)
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Set the number of threads for parallel generation (default: number of CPUs)
    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.config.num_threads = num_threads;
        self
    }

    /// Set Parquet compression format (default: SNAPPY)
    pub fn with_parquet_compression(mut self, compression: Compression) -> Self {
        self.config.parquet_compression = compression;
        self
    }

    /// Set target row group size in bytes for Parquet files (default: 7MB)
    pub fn with_parquet_row_group_bytes(mut self, bytes: i64) -> Self {
        self.config.parquet_row_group_bytes = bytes;
        self
    }

    /// Set the number of partitions to generate
    pub fn with_parts(mut self, parts: i32) -> Self {
        self.config.parts = Some(parts);
        self
    }

    /// Set the specific partition to generate (1-based, requires parts to be set)
    pub fn with_part(mut self, part: i32) -> Self {
        self.config.part = Some(part);
        self
    }

    /// Write output to stdout instead of files
    pub fn with_stdout(mut self, stdout: bool) -> Self {
        self.config.stdout = stdout;
        self
    }

    /// Set the CSV delimiter character (only applies to CSV format, default: ',')
    pub fn with_csv_delimiter(mut self, delimiter: char) -> Self {
        self.config.csv_delimiter = delimiter;
        self
    }

    /// Build the [`TpchGenerator`] with the configured settings
    pub fn build(self) -> TpchGenerator {
        TpchGenerator {
            config: self.config,
        }
    }
}

impl Default for TpchGeneratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
