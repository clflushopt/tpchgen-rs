use clap::Parser;
use tpcdsgen::config::Options;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let options = Options::parse();
    let session = options.to_session()?;
    tpcdsgen::dat::generate(&session)
}
