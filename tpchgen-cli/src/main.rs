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
//
// The main function is the entry point for the CLI and it uses the `clap` crate
// to parse the command line arguments and then generate the data.

// tpchgen-cli/src/main.rs
use clap::{Parser, ValueEnum};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSupplierGenerator, RegionGenerator, SupplierGenerator,
};

#[derive(Parser)]
#[command(name = "tpchgen")]
#[command(about = "TPC-H Data Generator", long_about = None)]
struct Cli {
    /// Scale factor to address defaults to 1.
    #[arg(short, long, default_value_t = 1.)]
    scale_factor: f64,

    /// Output directory for generated files
    #[arg(short, long)]
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

fn main() -> io::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&cli.output_dir)?;

    // Determine which tables to generate
    let tables: Vec<Table> = if let Some(tables) = cli.tables.as_ref() {
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

    // Generate each table
    for table in tables {
        match table {
            Table::Nation => generate_nation(&cli)?,
            Table::Region => generate_region(&cli)?,
            Table::Part => generate_part(&cli)?,
            Table::Supplier => generate_supplier(&cli)?,
            Table::PartSupp => generate_partsupp(&cli)?,
            Table::Customer => generate_customer(&cli)?,
            Table::Orders => generate_orders(&cli)?,
            Table::LineItem => generate_lineitem(&cli)?,
        }
    }

    println!("Generation complete!");
    Ok(())
}

fn new_table_writer(cli: &Cli, filename: &str) -> io::Result<BufWriter<File>> {
    let path = cli.output_dir.join(filename);
    let file = File::create(path)?;
    Ok(BufWriter::with_capacity(32 * 1024, file))
}

fn generate_nation(cli: &Cli) -> io::Result<()> {
    let filename = "nation.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = NationGenerator::new();
    for nation in generator.iter() {
        writeln!(writer, "{nation}",)?;
    }
    writer.flush()
}

fn generate_region(cli: &Cli) -> io::Result<()> {
    let filename = "region.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = RegionGenerator::new();
    for region in generator.iter() {
        writeln!(writer, "{region}",)?;
    }
    writer.flush()
}

fn generate_part(cli: &Cli) -> io::Result<()> {
    let filename = "part.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for part in generator.iter() {
        writeln!(writer, "{part}",)?;
    }
    writer.flush()
}

fn generate_supplier(cli: &Cli) -> io::Result<()> {
    let filename = "supplier.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = SupplierGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for supplier in generator.iter() {
        writeln!(writer, "{supplier}")?;
    }
    writer.flush()
}

fn generate_partsupp(cli: &Cli) -> io::Result<()> {
    let filename = "partsupp.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartSupplierGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for ps in generator.iter() {
        writeln!(writer, "{ps}")?;
    }
    writer.flush()
}

fn generate_customer(cli: &Cli) -> io::Result<()> {
    let filename = "customer.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = CustomerGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for customer in generator.iter() {
        writeln!(writer, "{customer}",)?;
    }
    writer.flush()
}

fn generate_orders(cli: &Cli) -> io::Result<()> {
    let filename = "orders.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = OrderGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for order in generator.iter() {
        writeln!(writer, "{order}")?;
    }
    writer.flush()
}

fn generate_lineitem(cli: &Cli) -> io::Result<()> {
    let filename = "lineitem.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = LineItemGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for item in generator.iter() {
        writeln!(writer, "{item}")?;
    }
    writer.flush()
}
