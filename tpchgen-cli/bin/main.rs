//! TPCH data generation CLI with a dbgen compatible API.
//!
//! This crate provides a CLI for generating TPCH data and tries to remain close
//! API wise to the original dbgen tool, as in we use the same command line flags
//! and arguments.
//!
//! See the documentation on [`Cli`] for more information on the command line

// Use the library public API
use clap::builder::TypedValueParser;
use clap::Parser;
use log::{info, LevelFilter};
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use tpchgen_cli::{
    Compression, OutputFormat, Table, TpchGenerator, TpchGeneratorBuilder,
    DEFAULT_PARQUET_ROW_GROUP_BYTES,
};

#[derive(Parser)]
#[command(name = "tpchgen")]
#[command(version)]
#[command(
    // -h output
    about = "TPC-H Data Generator",
    // --help output
    long_about = r#"
TPCH Data Generator (https://github.com/clflushopt/tpchgen-rs)

By default each table is written to a single file named <output_dir>/<table>.<format>

If `--part` option is specified, each table is written to a subdirectory in
multiple files named <output_dir>/<table>/<table>.<part>.<format>

Examples

# Generate all tables at scale factor 1 (1GB) in TBL format (default) to /tmp/tpch directory:

tpchgen-cli -s 1 --output-dir=/tmp/tpch

# Generate all tables in CSV format:

tpchgen-cli csv -s 1 --output-dir=/tmp/tpch

# Generate all tables in CSV format with tab delimiter:

tpchgen-cli csv -s 1 --delimiter='\t' --output-dir=/tmp/tpch

# Generate the lineitem table at scale factor 100 in 10 Apache Parquet files to
# /tmp/tpch/lineitem:

tpchgen-cli parquet -s 100 --tables=lineitem --parts=10 --output-dir=/tmp/tpch

# Generate scale factor one in current directory, seeing debug output

RUST_LOG=debug tpchgen-cli -s 1 --output-dir=/tmp/tpch
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    args: TopLevelArgs,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate TBL (pipe-delimited) output
    Tbl(TblArgs),
    /// Generate CSV output with CSV-specific options
    Csv(CsvArgs),
    /// Generate Apache Parquet output with Parquet-specific options
    Parquet(ParquetArgs),
}

#[derive(clap::Args)]
struct CommonArgs {
    /// Scale factor to create
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short = 'T', long = "tables", value_delimiter = ',', value_parser = TableValueParser)]
    tables: Option<Vec<Table>>,

    /// Number of part(itions) to generate. If not specified creates a single file per table
    #[arg(short, long)]
    parts: Option<i32>,

    /// Which part(ition) to generate (1-based). If not specified, generates all parts
    #[arg(long)]
    part: Option<i32>,

    /// The number of threads for parallel generation, defaults to the number of CPUs
    #[arg(short, long, default_value_t = num_cpus::get())]
    num_threads: usize,

    /// Verbose output
    ///
    /// When specified, sets the log level to `info` and ignores the `RUST_LOG`
    /// environment variable. When not specified, uses `RUST_LOG`
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet mode - only show error-level logs
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,

    /// Write the output to stdout instead of a file.
    #[arg(long, default_value_t = false)]
    stdout: bool,
}

impl CommonArgs {
    /// Create a [`TpchGeneratorBuilder`] pre-configured with the common options.
    fn builder(self, format: OutputFormat) -> TpchGeneratorBuilder {
        let mut builder = TpchGenerator::builder()
            .with_scale_factor(self.scale_factor)
            .with_output_dir(self.output_dir)
            .with_format(format)
            .with_num_threads(self.num_threads)
            .with_stdout(self.stdout);

        if let Some(tables) = self.tables {
            builder = builder.with_tables(tables);
        }
        if let Some(parts) = self.parts {
            builder = builder.with_parts(parts);
        }
        if let Some(part) = self.part {
            builder = builder.with_part(part);
        }

        builder
    }
}

#[derive(clap::Args)]
struct TopLevelArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// Output format (deprecated: use subcommands `tbl`, `csv`, or `parquet` instead)
    ///
    /// The --format flag will be removed in v4.0.0.
    #[arg(short, long, hide = true)]
    #[deprecated]
    format: Option<OutputFormat>,

    /// Parquet block compression format (deprecated: use 'parquet' subcommand instead)
    #[arg(short = 'c', long, hide = true)]
    #[deprecated]
    parquet_compression: Option<Compression>,

    /// Target row group size in bytes (deprecated: use 'parquet' subcommand instead)
    #[arg(long, hide = true)]
    #[deprecated]
    parquet_row_group_bytes: Option<i64>,
}

#[derive(clap::Args)]
struct TblArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(clap::Args)]
struct CsvArgs {
    #[command(flatten)]
    common: CommonArgs,

    /// CSV delimiter character (default: ',')
    ///
    /// Specifies the delimiter character to use when generating CSV files.
    ///
    /// Supports escape sequences: \t (tab), \n (newline), \r (carriage return), \\ (backslash)
    /// Common delimiters: ',' (comma), '|' (pipe), '\t' (tab), ';' (semicolon)
    #[arg(long, default_value = ",", value_parser = parse_delimiter)]
    delimiter: char,
}

#[derive(clap::Args)]
struct ParquetArgs {
    #[command(flatten)]
    common: CommonArgs,

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
    compression: Compression,

    /// Target size in row group bytes in Parquet files
    ///
    /// Row groups are the typical unit of parallel processing and compression
    /// with many query engines. Therefore, smaller row groups enable better
    /// parallelism and lower peak memory use but may reduce compression
    /// efficiency.
    ///
    /// Note: Parquet files are limited to 32k row groups, so at high scale
    /// factors, the row group size may be increased to keep the number of row
    /// groups under this limit.
    ///
    /// Typical values range from 10MB to 100MB.
    #[arg(long, default_value_t = DEFAULT_PARQUET_ROW_GROUP_BYTES)]
    row_group_bytes: i64,
}

/// Parse a delimiter string, handling escape sequences
fn parse_delimiter(s: &str) -> Result<char, String> {
    // Handle common escape sequences
    let parsed = match s {
        "\\t" => '\t',
        "\\n" => '\n',
        "\\r" => '\r',
        "\\\\" => '\\',
        _ => {
            // If it's not an escape sequence, it should be a single character
            let chars: Vec<char> = s.chars().collect();
            if chars.len() != 1 {
                return Err(format!(
                    "Delimiter must be a single character or escape sequence (\\t, \\n, \\r, \\\\), got: '{}'",
                    s
                ));
            }
            chars[0]
        }
    };
    Ok(parsed)
}

// TableValueParser is CLI-specific and uses the Table type from the library
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

#[tokio::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    cli.main().await
}

impl Cli {
    /// Main function to run the generation
    #[allow(deprecated)]
    async fn main(self) -> io::Result<()> {
        // Error if both --format and a subcommand are specified
        if self.args.format.is_some() && self.command.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot use --format with a subcommand. Use the subcommand directly, e.g. `tpchgen-cli parquet`",
            ));
        }
        match self.command {
            Some(Commands::Tbl(args)) => args.run().await,
            Some(Commands::Csv(args)) => args.run().await,
            Some(Commands::Parquet(args)) => args.run().await,
            None => self.run().await,
        }
    }

    #[allow(deprecated)]
    async fn run(self) -> io::Result<()> {
        let format = self.args.format.unwrap_or(OutputFormat::Tbl);

        configure_logging(self.args.common.verbose, self.args.common.quiet);

        // Warn about --format migration to subcommands (only when explicitly provided)
        if self.args.format.is_some() {
            match format {
                OutputFormat::Parquet => {
                    log::warn!("The --format flag will be removed in v4.0.0. Use `tpchgen-cli parquet` instead.");
                }
                OutputFormat::Csv => {
                    log::warn!("The --format flag will be removed in v4.0.0. Use `tpchgen-cli csv` instead.");
                }
                OutputFormat::Tbl => {
                    log::warn!("The --format flag will be removed in v4.0.0. Use `tpchgen-cli tbl` instead.");
                }
            }
        }

        let parquet_compression = self.args.parquet_compression.unwrap_or(Compression::SNAPPY);
        if self.args.parquet_compression.is_some() {
            if format == OutputFormat::Parquet {
                log::warn!("The --parquet-compression flag is deprecated. Use 'tpchgen-cli parquet --compression=...' instead");
            } else {
                log::warn!("Parquet compression option set but not generating Parquet files");
            }
        }

        let parquet_row_group_bytes = self
            .args
            .parquet_row_group_bytes
            .unwrap_or(DEFAULT_PARQUET_ROW_GROUP_BYTES);
        if self.args.parquet_row_group_bytes.is_some() {
            if format == OutputFormat::Parquet {
                log::warn!("The --parquet-row-group-bytes flag is deprecated. Use 'tpchgen-cli parquet --row-group-bytes=...' instead");
            } else {
                log::warn!("Parquet row group size option set but not generating Parquet files");
            }
        }

        self.args
            .common
            .builder(format)
            .with_parquet_compression(parquet_compression)
            .with_parquet_row_group_bytes(parquet_row_group_bytes)
            .build()
            .generate()
            .await
    }
}

impl TblArgs {
    async fn run(self) -> io::Result<()> {
        configure_logging(self.common.verbose, self.common.quiet);
        self.common
            .builder(OutputFormat::Tbl)
            .build()
            .generate()
            .await
    }
}

impl CsvArgs {
    async fn run(self) -> io::Result<()> {
        configure_logging(self.common.verbose, self.common.quiet);
        self.common
            .builder(OutputFormat::Csv)
            .with_csv_delimiter(self.delimiter)
            .build()
            .generate()
            .await
    }
}

impl ParquetArgs {
    async fn run(self) -> io::Result<()> {
        configure_logging(self.common.verbose, self.common.quiet);
        self.common
            .builder(OutputFormat::Parquet)
            .with_parquet_compression(self.compression)
            .with_parquet_row_group_bytes(self.row_group_bytes)
            .build()
            .generate()
            .await
    }
}

fn configure_logging(verbose: bool, quiet: bool) {
    if quiet {
        // Quiet mode: only show error-level logs
        env_logger::builder()
            .filter_level(LevelFilter::Error)
            .init();
    } else if verbose {
        env_logger::builder().filter_level(LevelFilter::Info).init();
        info!("Verbose output enabled (ignoring RUST_LOG environment variable)");
    } else {
        // Default: show warnings and errors, but respect RUST_LOG if set
        env_logger::builder()
            .filter_level(LevelFilter::Warn)
            .parse_default_env()
            .init();
    }
}
