//! TPC-DS data generation CLI with a dbgen compatible API.
use crate::tpch_cli::{Compression, DEFAULT_PARQUET_ROW_GROUP_BYTES};
use clap::{ArgAction, Args, Subcommand};
use std::fmt;
use std::path::PathBuf;
use tpcdsgen::config::CompatMode;
use tpcdsgen::config::Options as TpcdsOptions;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const NOT_IMPLEMENTED: &str = "TPC-DS data generation is not yet implemented";

#[derive(Args)]
#[command(version)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    args: DatArgs,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate DAT (pipe-delimited) output
    Dat(DatArgs),
    /// Generate CSV output with CSV-specific options
    Csv(CsvArgs),
    /// Generate Apache Parquet output with Parquet-specific options
    Parquet(ParquetArgs),
}

#[derive(Args)]
struct DatArgs {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Args)]
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

#[derive(Args)]
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

#[derive(Args)]
struct CommonArgs {
    /// Scale factor to create
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short = 'T', long = "tables", value_delimiter = ',')]
    tables: Option<Vec<String>>,

    /// Reference implementation to match (default: trino)
    #[arg(long, default_value_t = CompatMode::Trino)]
    compat: CompatMode,

    /// Verbose output
    ///
    /// When specified, sets the log level to `info` and ignores the `RUST_LOG`
    /// environment variable. When not specified, uses `RUST_LOG`
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet mode - only show error-level logs
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,

    /// Disable progress bars during data generation.
    ///
    /// Bars are also auto-suppressed by `--quiet` or when stderr is not a terminal.
    #[arg(long = "no-progress", action = ArgAction::SetFalse, default_value_t = true)]
    progress_bars_enabled: bool,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Some(Commands::Dat(args)) => args.run(),
            Some(Commands::Csv(args)) => args.run(),
            Some(Commands::Parquet(args)) => args.run(),
            None => self.args.run(),
        }
    }
}

impl DatArgs {
    fn run(self) -> Result<()> {
        self.common.run_dat()
    }
}

impl CsvArgs {
    fn run(self) -> Result<()> {
        let _ = self.delimiter;
        self.common.run_not_implemented()
    }
}

impl ParquetArgs {
    fn run(self) -> Result<()> {
        let _ = (self.compression, self.row_group_bytes);
        self.common.run_not_implemented()
    }
}

impl CommonArgs {
    fn run_dat(self) -> Result<()> {
        let _ = self.progress_bars_enabled;
        std::fs::create_dir_all(&self.output_dir)?;
        if let Some(tables) = &self.tables {
            for table in tables {
                self.run_dat_for_table(Some(table.clone()))?;
            }
        } else {
            self.run_dat_for_table(None)?;
        }

        Ok(())
    }

    fn run_dat_for_table(&self, table: Option<String>) -> Result<()> {
        let options = self.to_tpcds_options(table);
        let session = options.to_session()?;
        tpcdsgen::dat::generate(&session)
    }

    fn to_tpcds_options(&self, table: Option<String>) -> TpcdsOptions {
        let mut options = TpcdsOptions::new();
        options.scale = self.scale_factor;
        options.directory = self.output_dir.to_string_lossy().into_owned();
        options.table = table;
        options.compat = self.compat;
        options
    }

    fn run_not_implemented(self) -> Result<()> {
        let _ = self;
        Err(Box::new(NotImplemented))
    }
}

struct NotImplemented;

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(NOT_IMPLEMENTED)
    }
}

impl fmt::Debug for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for NotImplemented {}

fn parse_delimiter(s: &str) -> std::result::Result<char, String> {
    let parsed = match s {
        "\\t" => '\t',
        "\\n" => '\n',
        "\\r" => '\r',
        "\\\\" => '\\',
        _ => {
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
    if !parsed.is_ascii() {
        return Err(format!(
            "Delimiter must be an ASCII character, got: '{}'",
            parsed
        ));
    }
    Ok(parsed)
}
