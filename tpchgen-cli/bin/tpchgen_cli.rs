use clap::Parser;
use std::process::ExitCode;
use tpcgen_cli::tpch_cli::Cli;

// Same arguments, subcommands and help as `tpcgen-cli tpch`, plus this
// package's own `-V`/`--version`.
#[derive(Parser)]
#[command(
    name = "tpchgen-cli",
    version,
    mut_subcommands = format_long_about
)]
struct TpchgenCli {
    #[command(flatten)]
    cli: Cli,
}

/// Rewrite the `tpcgen-cli tpch` examples in a format's `--help` text for
/// `tpchgen-cli`
fn format_long_about(cmd: clap::Command) -> clap::Command {
    match cmd.get_long_about() {
        Some(text) => {
            let text = text.to_string().replace("tpcgen-cli tpch", "tpchgen-cli");
            cmd.long_about(text)
        }
        None => cmd,
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match TpchgenCli::parse().cli.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
