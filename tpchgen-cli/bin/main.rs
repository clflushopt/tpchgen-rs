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
    Compression, OutputFormat, Table, TpchGenerator, DEFAULT_PARQUET_ROW_GROUP_BYTES,
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

# Generate all tables at scale factor 1 (1GB) in TBL format to /tmp/tpch directory:

tpchgen-cli -s 1 --output-dir=/tmp/tpch

# Generate the lineitem table at scale factor 100 in 10 Apache Parquet files to
# /tmp/tpch/lineitem

tpchgen-cli -s 100 --tables=lineitem --format=parquet --parts=10 --output-dir=/tmp/tpch

# Generate scale factor one in current directory, seeing debug output

RUST_LOG=debug tpchgen -s 1
"#
)]
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

    /// Number of part(itions) to generate. If not specified creates a single file per table
    #[arg(short, long)]
    parts: Option<i32>,

    /// Which part(ition) to generate (1-based). If not specified, generates all parts
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
    ///
    /// When specified, sets the log level to `info` and ignores the `RUST_LOG`
    /// environment variable. When not specified, uses `RUST_LOG`
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Write the output to stdout instead of a file.
    #[arg(long, default_value_t = false)]
    stdout: bool,

    /// Target size in row group bytes in Parquet files
    ///
    /// Row groups are the typical unit of parallel processing and compression
    /// with many query engines. Therfore, smaller row groups enable better
    /// parallelism and lower peak memory use but may reduce compression
    /// efficiency.
    ///
    /// Note: Parquet files are limited to 32k row groups, so at high scale
    /// factors, the row group size may be increased to keep the number of row
    /// groups under this limit.
    ///
    /// Typical values range from 10MB to 100MB.
    #[arg(long, default_value_t = DEFAULT_PARQUET_ROW_GROUP_BYTES)]
    parquet_row_group_bytes: i64,
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
    async fn main(self) -> io::Result<()> {
        // Configure logging
        if self.verbose {
            env_logger::builder().filter_level(LevelFilter::Info).init();
            info!("Verbose output enabled (ignoring RUST_LOG environment variable)");
        } else {
            env_logger::init();
        }

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

        // Build the generator using the library API
        let mut builder = TpchGenerator::builder()
            .with_scale_factor(self.scale_factor)
            .with_output_dir(self.output_dir)
            .with_format(self.format)
            .with_num_threads(self.num_threads)
            .with_parquet_compression(self.parquet_compression)
            .with_parquet_row_group_bytes(self.parquet_row_group_bytes)
            .with_stdout(self.stdout);

        // Add tables if specified
        if let Some(tables) = self.tables {
            builder = builder.with_tables(tables);
        }

        // Add parts/part if specified
        if let Some(parts) = self.parts {
            builder = builder.with_parts(parts);
        }
        if let Some(part) = self.part {
            builder = builder.with_part(part);
        }

        // Generate using the library
        builder.build().generate().await
    }
}
