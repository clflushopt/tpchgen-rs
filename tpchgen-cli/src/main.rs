//! TPCH data generation CLI with a dbgen compatible API.
//!
//! This crate provides a CLI for generating TPCH data and tries to remain close
//! API wise to the original dbgen tool, as in we use the same command line flags
//! and arguments.
//!
//! -h, --help       Prints help information
//! -V, --version    Prints version information
//! -s, --scale      Scale factor for the data generation
//! -T, --tables     Tables to generate data for
//! -F, --format     Output format for the data (CSV or Parquet)
//! -O, --output     Output directory for the generated data
//! -v, --verbose    Verbose output
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
mod generate;
mod sources;

use crate::generate::{generate_in_chunks, Sink, Source};
use crate::sources::*;
use clap::{Parser, ValueEnum};
use log::{debug, info, LevelFilter};
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;
use tpchgen::distribution::Distributions;
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSupplierGenerator, RegionGenerator, SupplierGenerator,
};
use tpchgen::text::TextPool;

#[derive(Parser)]
#[command(name = "tpchgen")]
#[command(about = "TPC-H Data Generator", long_about = None)]
struct Cli {
    /// Scale factor to address (default: 1)
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Which tables to generate (default: all)
    #[arg(short, long)]
    tables: Option<Vec<Table>>,

    /// Number of parts to generate (for parallel generation)
    #[arg(short, long, default_value_t = 1)]
    parts: i32,

    /// Which part to generate (1-based, only relevant if parts > 1)
    #[arg(long, default_value_t = 1)]
    part: i32,

    /// Output format: tbl, csv, parquet (default: tbl)
    #[arg(short, long, default_value = "tbl")]
    format: OutputFormat,

    /// Verbose output (default: false)
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Table {
    Nation,
    Region,
    Part,
    Supplier,
    PartSupp,
    Customer,
    Orders,
    LineItem,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Table {
    fn name(&self) -> &'static str {
        match self {
            Table::Nation => "nation",
            Table::Region => "region",
            Table::Part => "part",
            Table::Supplier => "supplier",
            Table::PartSupp => "partsupp",
            Table::Customer => "customer",
            Table::Orders => "orders",
            Table::LineItem => "lineitem",
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

/// macro to create a Cli function for generating a table
///
/// Arguments:
/// $FUN_NAME: name of the function to create
/// $TABLE: The [`Table`] to generate
/// $GENERATOR: The generator type to use
/// $TBL_SOURCE: The [`Source`] type to use for TBL format
/// $CSV_SOURCE: The [`Source`] type to use for CSV format
macro_rules! define_generate {
    ($FUN_NAME:ident,  $TABLE:expr, $GENERATOR:ident, $TBL_SOURCE:ty, $CSV_SOURCE:ty) => {
        async fn $FUN_NAME(&self) -> io::Result<()> {
            let filename = self.output_filename($TABLE);
            let (num_parts, parts) = self.parallel_target_part_count(&$TABLE);
            let scale_factor = self.scale_factor;
            info!("Writing table {} (SF={scale_factor}) to {filename}", $TABLE);
            debug!("Generating {num_parts} parts in total");
            let gens = parts
                .into_iter()
                .map(move |part| $GENERATOR::new(scale_factor, part, num_parts));
            match self.format {
                OutputFormat::Tbl => self.go(&filename, gens.map(<$TBL_SOURCE>::new)).await,
                OutputFormat::Csv => self.go(&filename, gens.map(<$CSV_SOURCE>::new)).await,
                // https://github.com/clflushopt/tpchgen-rs/issues/46
                OutputFormat::Parquet => unimplemented!("Parquet support not yet implemented"),
            }
        }
    };
}

impl Cli {
    async fn main(self) -> io::Result<()> {
        if self.verbose {
            // explicitly set logging to info / stdout
            env_logger::builder().filter_level(LevelFilter::Info).init();
            info!("Verbose output enabled (ignoring RUST_LOG environment variable)");
        } else {
            env_logger::init();
            debug!("Logging configured from environment variables");
        }

        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Determine which tables to generate
        let tables: Vec<Table> = if let Some(tables) = self.tables.as_ref() {
            tables.clone()
        } else {
            vec![
                Table::Nation,
                Table::Region,
                Table::Part,
                Table::Supplier,
                Table::PartSupp,
                Table::Customer,
                Table::Orders,
                Table::LineItem,
            ]
        };

        // force the creation of the distributions and text pool to so it doesn't
        // get charged to the first table
        let start = Instant::now();
        debug!("Creating distributions and text pool");
        Distributions::static_default();
        TextPool::get_or_init_default();
        let elapsed = start.elapsed();
        info!("Created static distributions and text pools in {elapsed:?}");

        // Generate each table
        for table in tables {
            match table {
                Table::Nation => self.generate_nation().await?,
                Table::Region => self.generate_region().await?,
                Table::Part => self.generate_part().await?,
                Table::Supplier => self.generate_supplier().await?,
                Table::PartSupp => self.generate_partsupp().await?,
                Table::Customer => self.generate_customer().await?,
                Table::Orders => self.generate_orders().await?,
                Table::LineItem => self.generate_lineitem().await?,
            }
        }

        info!("Generation complete!");
        Ok(())
    }

    define_generate!(
        generate_nation,
        Table::Nation,
        NationGenerator,
        NationTblSource,
        NationCsvSource
    );
    define_generate!(
        generate_region,
        Table::Region,
        RegionGenerator,
        RegionTblSource,
        RegionCsvSource
    );
    define_generate!(
        generate_part,
        Table::Part,
        PartGenerator,
        PartTblSource,
        PartCsvSource
    );
    define_generate!(
        generate_supplier,
        Table::Supplier,
        SupplierGenerator,
        SupplierTblSource,
        SupplierCsvSource
    );
    define_generate!(
        generate_partsupp,
        Table::PartSupp,
        PartSupplierGenerator,
        PartSuppTblSource,
        PartSuppCsvSource
    );
    define_generate!(
        generate_customer,
        Table::Customer,
        CustomerGenerator,
        CustomerTblSource,
        CustomerCsvSource
    );
    define_generate!(
        generate_orders,
        Table::Orders,
        OrderGenerator,
        OrderTblSource,
        OrderCsvSource
    );
    define_generate!(
        generate_lineitem,
        Table::LineItem,
        LineItemGenerator,
        LineItemTblSource,
        LineItemCsvSource
    );

    /// return the output filename for the given table
    fn output_filename(&self, table: Table) -> String {
        let extension = match self.format {
            OutputFormat::Tbl => "tbl",
            OutputFormat::Csv => "csv",
            OutputFormat::Parquet => "parquet",
        };
        format!("{}.{extension}", table.name())
    }

    /// return a buffered file for writing the given filename in the output directory
    fn new_output_writer(&self, filename: &str) -> io::Result<BufWriter<File>> {
        let path = self.output_dir.join(filename);
        let file = File::create(path)?;
        Ok(BufWriter::with_capacity(32 * 1024, file))
    }

    /// Returns a list of "parts" (data generator chunks, not TPCH parts) to create
    ///
    /// Tuple returned is `(num_parts, part_list)`:
    /// - num_parts is the total number of parts to generate
    /// - part_list is the list of parts to generate (1 based)
    fn parallel_target_part_count(&self, table: &Table) -> (i32, Vec<i32>) {
        // parallel generation disabled if user specifies a part explicitly
        if self.part != 1 || self.parts != 1 {
            return (self.parts, vec![self.part]);
        }

        // Note use part=1, part_count=1 to calculate the total row count
        // for the table
        //
        // Avg row size is an estimate of the average row size in bytes from the first 100 rows
        // of the table in tbl format
        let (avg_row_size_bytes, row_count) = match table {
            Table::Nation => (88, 1),
            Table::Region => (77, 1),
            Table::Part => (
                115,
                PartGenerator::calculate_row_count(self.scale_factor, 1, 1),
            ),
            Table::Supplier => (
                140,
                SupplierGenerator::calculate_row_count(self.scale_factor, 1, 1),
            ),
            Table::PartSupp => (
                148,
                PartSupplierGenerator::calculate_row_count(self.scale_factor, 1, 1),
            ),
            Table::Customer => (
                160,
                CustomerGenerator::calculate_row_count(self.scale_factor, 1, 1),
            ),
            Table::Orders => (
                114,
                OrderGenerator::calculate_row_count(self.scale_factor, 1, 1),
            ),
            Table::LineItem => {
                // there are on average 4 line items per order.
                // For example, in SF=10,
                // * orders has 15,000,000 rows
                // * lineitem has around 60,000,000 rows
                let row_count = 4 * OrderGenerator::calculate_row_count(self.scale_factor, 1, 1);
                (128, row_count)
            }
        };
        // target chunks of about 16MB (use 15MB to ensure we don't exceed the target size)
        let target_chunk_size_bytes = 15 * 1024 * 1024;
        let num_parts = ((row_count * avg_row_size_bytes) / target_chunk_size_bytes) + 1;
        // convert to i32
        let num_parts = num_parts.try_into().unwrap();
        // generating all the parts
        (num_parts, (1..=num_parts).collect())
    }

    /// Generates the output file from the sources
    async fn go<I>(&self, filename: &str, sources: I) -> Result<(), io::Error>
    where
        I: Iterator<Item: Source> + 'static,
    {
        let sink = BufWriterSink::new(self.new_output_writer(filename)?);
        generate_in_chunks(sink, sources).await
    }
}

/// Wrapper around a buffer writer that counts the number of buffers and bytes written
struct BufWriterSink {
    start: Instant,
    inner: BufWriter<File>,
    num_buffers: usize,
    num_bytes: usize,
}

impl BufWriterSink {
    fn new(inner: BufWriter<File>) -> Self {
        Self {
            start: Instant::now(),
            inner,
            num_buffers: 0,
            num_bytes: 0,
        }
    }
}

impl Sink for BufWriterSink {
    fn sink(&mut self, buffer: &[u8]) -> Result<(), io::Error> {
        self.num_buffers += 1;
        self.num_bytes += buffer.len();
        self.inner.write_all(buffer)
    }

    fn finish(mut self) -> Result<(), io::Error> {
        let res = self.inner.flush();

        let duration = self.start.elapsed();
        let mb_per_buffer = self.num_bytes as f64 / (1024.0 * 1024.0) / self.num_buffers as f64;
        let bytes_per_second = (self.num_bytes as f64 / duration.as_secs_f64()) as u64;
        let gb_per_second = bytes_per_second as f64 / (1024.0 * 1024.0 * 1024.0);

        info!("Completed in {duration:?} ({gb_per_second:.02} GB/sec)");
        debug!(
            "wrote {} bytes in {} buffers {mb_per_buffer:.02} MB/buffer",
            self.num_bytes, self.num_buffers,
        );
        res
    }
}
