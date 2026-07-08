//! TPC-DS data generation CLI with a dbgen compatible API.
use crate::logging::configure_logging;
#[cfg(feature = "indicatif-progress")]
use crate::progress::IndicatifProgress;
use crate::progress::{no_op_progress_tracker, ProgressTracker};
use crate::tpch_cli::{Compression, DEFAULT_PARQUET_ROW_GROUP_BYTES};
use clap::{ArgAction, Args, Subcommand};
use std::io;
#[cfg(feature = "indicatif-progress")]
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{CompatMode, Session, SessionBuilder, Table};
use tpcdsgen::error::TpcdsError;

pub mod csv;
pub mod dat;
pub mod parquet;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_TPCDS_PARQUET_ROW_GROUP_BYTES: usize = DEFAULT_PARQUET_ROW_GROUP_BYTES as usize;

enum OutputFormat {
    Dat(dat::Dat),
    Csv(csv::Csv),
    Parquet(parquet::Parquet),
}

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
    #[arg(
        long,
        default_value_t = DEFAULT_TPCDS_PARQUET_ROW_GROUP_BYTES,
        value_parser = parse_row_group_bytes
    )]
    row_group_bytes: usize,
}

#[derive(Args)]
pub struct CommonArgs {
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
        self.common.run_csv(self.delimiter)
    }
}

impl ParquetArgs {
    fn run(self) -> Result<()> {
        self.common
            .run_parquet(self.compression, self.row_group_bytes)
    }
}

impl CommonArgs {
    fn run_dat(self) -> Result<()> {
        let output = dat::Dat::new(self.output_dir.clone())?;
        let tables = self.dat_tables()?;
        // DAT return tables are emitted as side effects of their sales table generator.
        // CSV and Parquet have direct return-table generators and do not need expansion.
        self.run_output_with_tables(OutputFormat::Dat(output), tables)
    }

    fn run_parquet(self, compression: Compression, row_group_bytes: usize) -> Result<()> {
        let output = parquet::Parquet::new(self.output_dir.clone(), compression, row_group_bytes);
        self.run_output(OutputFormat::Parquet(output))
    }

    fn run_csv(self, delimiter: char) -> Result<()> {
        let output = csv::Csv::new(self.output_dir.clone(), delimiter);
        self.run_output(OutputFormat::Csv(output))
    }

    fn run_output(self, output_format: OutputFormat) -> Result<()> {
        let tables = self.tables()?;
        self.run_output_with_tables(output_format, tables)
    }

    fn run_output_with_tables(self, output_format: OutputFormat, tables: Vec<Table>) -> Result<()> {
        let (progress, log_writer) = self.progress_tracker();
        configure_logging(self.verbose, self.quiet, log_writer);

        std::fs::create_dir_all(&self.output_dir)?;

        for table in tables {
            let session = self.to_session(Some(table.get_name().to_string()))?;
            output_format.generate_table(table, session, progress.as_ref())?;
        }

        progress.finish();
        Ok(())
    }

    fn progress_tracker(
        &self,
    ) -> (
        Arc<dyn ProgressTracker>,
        Option<Box<dyn io::Write + Send + 'static>>,
    ) {
        #[cfg(feature = "indicatif-progress")]
        if self.progress_bars_enabled && !self.quiet && io::stderr().is_terminal() {
            let progress = Arc::new(IndicatifProgress::new());
            let tracker: Arc<dyn ProgressTracker> = progress.clone();
            return (tracker, Some(progress.log_writer()));
        }

        (no_op_progress_tracker(), None)
    }

    /// Return the tables that should be generated.
    fn tables(&self) -> Result<Vec<Table>> {
        if let Some(tables) = &self.tables {
            tables.iter().map(|table| parse_table(table)).collect()
        } else {
            Ok(Table::main_tables())
        }
    }

    /// Return the DAT tables to generate, mapping return-only selections to
    /// their sales table generators because DAT emits return files as side effects.
    fn dat_tables(&self) -> Result<Vec<Table>> {
        let mut tables = Vec::new();
        for table in self.tables()? {
            let table = match table {
                Table::CatalogReturns => Table::CatalogSales,
                Table::StoreReturns => Table::StoreSales,
                Table::WebReturns => Table::WebSales,
                table => table,
            };
            if !tables.contains(&table) {
                tables.push(table);
            }
        }
        Ok(tables)
    }

    fn to_session(&self, table: Option<String>) -> Result<Session> {
        let table = table.as_deref().map(parse_table).transpose()?;

        // store the command line arguments used to create this
        let command_line_arguments = std::env::args().collect::<Vec<_>>().join(" ");

        let mut builder = SessionBuilder::new()
            .with_scale_factor(self.scale_factor)
            .with_compat_mode(self.compat)
            .with_command_line_arguments(command_line_arguments);

        if let Some(table) = table {
            builder = builder.with_table(table);
        }

        Ok(builder.build()?)
    }
}

impl OutputFormat {
    fn generate_table(
        &self,
        table: Table,
        session: Session,
        progress: &dyn ProgressTracker,
    ) -> Result<()> {
        match self {
            OutputFormat::Dat(output) => output.generate_table(table, &session, progress),
            OutputFormat::Csv(output) => output.generate_table(table, session, progress),
            OutputFormat::Parquet(output) => output.generate_table(table, session, progress),
        }
    }
}

fn parse_table(table: &str) -> Result<Table> {
    let parsed = table.parse::<Table>().map_err(|_| {
        TpcdsError::new(&format!(
            "unknown table '{table}'. Expected one of: {}",
            expected_table_names()
        ))
    })?;

    if parsed.is_main_table() {
        Ok(parsed)
    } else {
        Err(TpcdsError::new(&format!(
            "unknown table '{table}'. Expected one of: {}",
            expected_table_names()
        ))
        .into())
    }
}

fn expected_table_names() -> String {
    Table::main_tables()
        .iter()
        .map(Table::get_name)
        .collect::<Vec<_>>()
        .join(", ")
}

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

fn parse_row_group_bytes(s: &str) -> std::result::Result<usize, String> {
    let parsed = s.parse::<usize>().map_err(|e| e.to_string())?;
    if parsed == 0 {
        Err("must be greater than zero".to_string())
    } else {
        Ok(parsed)
    }
}
