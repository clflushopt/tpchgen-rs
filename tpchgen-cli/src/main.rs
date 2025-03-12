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

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(version, about = "TPCH data generation CLI with a dbgen compatible API.", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Scale factor to use can be one of (0.01, 0.1, 1, 10, 30, 100, 300, 1000, 3000"
    )]
    scale: f64,

    #[arg(
        short,
        help = "Generate data for only the specified tables (defaults to all)."
    )]
    tables: Vec<String>,

    #[arg(short, long, help = "Output format for the data (CSV or Parquet)")]
    format: String,

    #[arg(short, long, help = "Output directory for the generated data")]
    output: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}
