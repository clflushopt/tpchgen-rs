//! TPCH data generation CLI and library API.

mod cli;
mod generator;

pub mod csv;
pub mod generate;
pub mod output_plan;
pub mod parquet;
pub mod plan;
#[cfg(not(feature = "progress"))]
mod progress;
#[cfg(feature = "progress")]
pub mod progress;
pub mod runner;
pub mod statistics;
pub mod tbl;

pub use cli::Cli;
pub use generator::{
    Compression, GeneratorConfig, OutputFormat, Table, TpchGenerator, TpchGeneratorBuilder,
    WriterSink,
};
pub use plan::{GenerationPlan, DEFAULT_PARQUET_ROW_GROUP_BYTES};
