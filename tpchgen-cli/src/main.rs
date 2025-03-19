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

use std::collections::VecDeque;
// tpchgen-cli/src/main.rs
use clap::{Parser, ValueEnum};
use futures::StreamExt;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tokio::task::JoinSet;
use tpchgen::dates::GenerateUtils;
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

#[tokio::main]
async fn main() -> io::Result<()> {
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
            Table::LineItem => generate_lineitem(&cli).await?,
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
        writeln!(
            writer,
            "{}|{}|{}|{}|",
            nation.n_nationkey, nation.n_name, nation.n_regionkey, nation.n_comment
        )?;
    }
    writer.flush()
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
    writer.flush()
}

fn generate_part(cli: &Cli) -> io::Result<()> {
    let filename = "part.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartGenerator::new(cli.scale_factor, cli.part, cli.parts);
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
    writer.flush()
}

fn generate_supplier(cli: &Cli) -> io::Result<()> {
    let filename = "supplier.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = SupplierGenerator::new(cli.scale_factor, cli.part, cli.parts);
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
    writer.flush()
}

fn generate_partsupp(cli: &Cli) -> io::Result<()> {
    let filename = "partsupp.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = PartSupplierGenerator::new(cli.scale_factor, cli.part, cli.parts);
    for ps in generator.iter() {
        writeln!(
            writer,
            "{}|{}|{}|{:.2}|{}|",
            ps.ps_partkey, ps.ps_suppkey, ps.ps_availqty, ps.ps_supplycost, ps.ps_comment
        )?;
    }
    writer.flush()
}

fn generate_customer(cli: &Cli) -> io::Result<()> {
    let filename = "customer.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = CustomerGenerator::new(cli.scale_factor, cli.part, cli.parts);
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
    writer.flush()
}

fn generate_orders(cli: &Cli) -> io::Result<()> {
    let filename = "orders.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let generator = OrderGenerator::new(cli.scale_factor, cli.part, cli.parts);
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
    writer.flush()
}

async fn generate_lineitem(cli: &Cli) -> io::Result<()> {
    let filename = "lineitem.tbl";
    let mut writer = new_table_writer(cli, filename)?;

    let start = Instant::now();
    // figure out how to evenly divide the workload
    let total_orders = GenerateUtils::calculate_row_count(
        OrderGenerator::SCALE_BASE,
        cli.scale_factor,
        cli.part,
        cli.parts,
    );
    // each order has some random number of line items between 1 and 7
    // We target 10000 orders for an average 35000 line items per part
    let num_parts = (total_orders / 10000) + 1;
    let num_parts: i32 = num_parts.try_into().unwrap();
    println!("Total orders: {total_orders}, num parts: {num_parts}");

    let recycler = BufferRecycler::new();
    let captured_recycler = &recycler;
    // use all cores to make dta
    let num_tasks = num_cpus::get() - 1;
    println!("Using {num_tasks} parallel tasks");

    let generators =
        (0..num_parts).map(|part0| LineItemGenerator::new(cli.scale_factor, part0 + 1, num_parts));
    // convert to an async stream to run on tokio
    let mut stream = futures::stream::iter(generators)
        // each job will generate a buffer holding its part  of the lineitem table
        .map(async move |generator| {
            let buffer = captured_recycler.new_buffer(1024 * 1024 * 8);
            generate_buffer(generator, buffer).await
        })
        // run in parallel
        .buffered(num_tasks);

    // write it to the output as fast as we can go
    let mut num_buffers = 0;
    let mut num_bytes = 0;
    while let Some(buffer) = stream.next().await {
        //println!("writing buffer with {} bytes", buffer.len());
        num_buffers += 1;
        num_bytes += buffer.len();
        writer.write_all(&buffer)?;
        recycler.return_buffer(buffer);
    }
    let duration = start.elapsed();
    let bytes_per_second = (num_bytes as f64 / duration.as_secs_f64()) as u64;
    let gb_per_second = bytes_per_second as f64 / (1024.0 * 1024.0 * 1024.0);
    let res = writer.flush();
    println!("wrote {num_buffers} buffers, {num_bytes} bytes ({gb_per_second:02} GB bytes/sec)");
    res
}

/// Generate a buffer of data on a different async task using the specified generator
/// and return the results
async fn generate_buffer(generator: LineItemGenerator<'static>, mut buffer: Vec<u8>) -> Vec<u8> {
    let mut join_set = JoinSet::new();
    // do the work in a task
    join_set.spawn(async move {
        for item in generator.iter() {
            writeln!(
                &mut buffer,
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
            )
            .expect("writing to in memory buffer is infallable");
        }
        buffer
    });
    // wait for the task to be done and return the result
    let mut buffers = join_set.join_all().await;
    assert_eq!(buffers.len(), 1);
    buffers.pop().unwrap()
}

/// A simple buffer recycler to avoid allocating new buffers for each part
struct BufferRecycler {
    buffers: Mutex<VecDeque<Vec<u8>>>,
}

impl BufferRecycler {
    fn new() -> Self {
        Self {
            buffers: Mutex::new(VecDeque::new()),
        }
    }
    /// return a new empty buffer, with size bytes capacity
    fn new_buffer(&self, size: usize) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            if size > buffer.capacity() {
                buffer.reserve(size - buffer.capacity());
            }
            buffer
        } else {
            Vec::with_capacity(size)
        }
    }

    fn return_buffer(&self, buffer: Vec<u8>) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.push_back(buffer);
    }
}
