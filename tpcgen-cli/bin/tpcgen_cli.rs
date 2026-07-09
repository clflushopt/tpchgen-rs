//! Binary entry point for the TPC-H and TPC-DS data generator.

use clap::{Parser, Subcommand};
use std::process::ExitCode;
use tpcgen_cli::tpcds_cli::Cli as TpcdsCli;
use tpcgen_cli::tpch_cli::Cli as TpchCli;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "tpcgen-cli")]
#[command(version)]
#[command(
    about = "TPC-H and TPC-DS data generator",
    long_about = r#"
TPC-H and TPC-DS data generator (https://github.com/clflushopt/tpchgen-rs)

Examples

# TPC-H TBL data:

tpcgen-cli tpch -s 1 --output-dir=/tmp/tpch

# TPC-H CSV data:

tpcgen-cli tpch csv -s 1 --output-dir=/tmp/tpch

# TPC-H Apache Parquet data:

tpcgen-cli tpch parquet -s 100 --tables=lineitem --parts=10 --output-dir=/tmp/tpch
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// TPC-H data
    Tpch(TpchCli),
    /// TPC-DS data
    Tpcds(TpcdsCli),
}

impl Cli {
    async fn run(self) -> Result<()> {
        match self.command {
            Command::Tpch(args) => args.run().await?,
            Command::Tpcds(args) => args.run().await?,
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match Cli::parse().run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
