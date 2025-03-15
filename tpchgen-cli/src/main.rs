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
        println!("Generating table: {:?}", table);
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

fn new_table_writer(cli: &Cli, filename: &str) -> io::Result<Box<dyn Write>> {
    let path = cli.output_dir.join(filename);
    let file = File::create(path)?;

    Ok(Box::new(BufWriter::new(file)))
}

fn generate_nation(cli: &Cli) -> io::Result<()> {
    let filename = "nation.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = NationGenerator::new();
    for nation in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{}|",
            nation.n_nationkey, nation.n_name, nation.n_regionkey, nation.n_comment
        )?;
    }

    Ok(())
}

fn generate_region(cli: &Cli) -> io::Result<()> {
    let filename = "region.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = RegionGenerator::new();
    for region in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|",
            region.r_regionkey, region.r_name, region.r_comment
        )?;
    }

    Ok(())
}

fn generate_part(cli: &Cli) -> io::Result<()> {
    let filename = "part.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for part in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{}|{}|{}|{}|{:.2}|{}|",
            part.p_partkey,
            part.p_name,
            part.p_mfgr,
            part.p_brand,
            part.p_type,
            part.p_size,
            part.p_container,
            part.p_retailprice,
            part.p_comment
        )?;
    }

    Ok(())
}

fn generate_supplier(cli: &Cli) -> io::Result<()> {
    let filename = "supplier.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = SupplierGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for supplier in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{}|{}|{:.2}|{}|",
            supplier.s_suppkey,
            supplier.s_name,
            supplier.s_address,
            supplier.s_nationkey,
            supplier.s_phone,
            supplier.s_acctbal,
            supplier.s_comment
        )?;
    }

    Ok(())
}

fn generate_partsupp(cli: &Cli) -> io::Result<()> {
    let filename = "partsupp.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartSupplierGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for ps in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{:.2}|{}|",
            ps.ps_partkey, ps.ps_suppkey, ps.ps_availqty, ps.ps_supplycost, ps.ps_comment
        )?;
    }

    Ok(())
}

fn generate_customer(cli: &Cli) -> io::Result<()> {
    let filename = "customer.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = CustomerGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for customer in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{}|{}|{:.2}|{}|{}|",
            customer.c_custkey,
            customer.c_name,
            customer.c_address,
            customer.c_nationkey,
            customer.c_phone,
            customer.c_acctbal,
            customer.c_mktsegment,
            customer.c_comment
        )?;
    }

    Ok(())
}

fn generate_orders(cli: &Cli) -> io::Result<()> {
    let filename = "orders.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = OrderGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for order in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{:.2}|{}|{}|{}|{}|{}|",
            order.o_orderkey,
            order.o_custkey,
            order.o_orderstatus,
            order.o_totalprice,
            order.o_orderdate,
            order.o_orderpriority,
            order.o_clerk,
            order.o_shippriority,
            order.o_comment
        )?;
    }

    Ok(())
}

fn generate_lineitem(cli: &Cli) -> io::Result<()> {
    let filename = "lineitem.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = LineItemGenerator::new(cli.scale_factor as f64, cli.part, cli.parts);
    for item in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{}|{:.2}|{:.2}|{:.2}|{:.2}|{}|{}|{}|{}|{}|{}|{}|{}|",
            item.l_orderkey,
            item.l_partkey,
            item.l_suppkey,
            item.l_linenumber,
            item.l_quantity,
            item.l_extendedprice,
            item.l_discount,
            item.l_tax,
            item.l_returnflag,
            item.l_linestatus,
            item.l_shipdate,
            item.l_commitdate,
            item.l_receiptdate,
            item.l_shipinstruct,
            item.l_shipmode,
            item.l_comment
        )?;
    }

    Ok(())
}
