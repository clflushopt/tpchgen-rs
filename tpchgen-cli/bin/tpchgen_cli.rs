use clap::Parser;
use std::io;
use tpcgen::tpch_cli::Cli;

#[tokio::main]
async fn main() -> io::Result<()> {
    Cli::parse().run().await
}
