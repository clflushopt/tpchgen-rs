//! [`PlanRunner`] for running [`OutputPartitionPlan`]s.

use crate::csv::*;
use crate::generate::{generate_in_chunks, Source};
use crate::parquet::generate_parquet;
use crate::plan::{OutputLocation, OutputPartitionPlan};
use crate::tbl::*;
use crate::tbl::{LineItemTblSource, NationTblSource, RegionTblSource};
use crate::{OutputFormat, Table, WriterSink};
use log::{debug, info};
use std::io;
use std::io::BufWriter;
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSuppGenerator, RegionGenerator, SupplierGenerator,
};
use tpchgen_arrow::{
    CustomerArrow, LineItemArrow, NationArrow, OrderArrow, PartArrow, PartSuppArrow,
    RecordBatchIterator, RegionArrow, SupplierArrow,
};

#[derive(Debug)]
pub struct PlanRunner {
    plans: Vec<OutputPartitionPlan>,
    num_threads: usize,
}

impl PlanRunner {
    pub fn new(plans: Vec<OutputPartitionPlan>, num_threads: usize) -> Self {
        Self { plans, num_threads }
    }

    /// Run all the plans in the runner.
    pub async fn run(self) -> Result<(), io::Error> {
        debug!(
            "Running {} plans with {} threads...",
            self.plans.len(),
            self.num_threads
        );
        let Self { plans, num_threads } = self;

        for plan in plans {
            run_plan(plan, num_threads).await?;
        }
        Ok(())
    }
}

/// Run a single [`OutputPartitionPlan`]
async fn run_plan(plan: OutputPartitionPlan, num_threads: usize) -> io::Result<()> {
    match plan.table() {
        Table::Nation => run_nation_plan(plan, num_threads).await,
        Table::Region => run_region_plan(plan, num_threads).await,
        Table::Part => run_part_plan(plan, num_threads).await,
        Table::Supplier => run_supplier_plan(plan, num_threads).await,
        Table::Partsupp => run_partsupp_plan(plan, num_threads).await,
        Table::Customer => run_customer_plan(plan, num_threads).await,
        Table::Orders => run_orders_plan(plan, num_threads).await,
        Table::Lineitem => run_lineitem_plan(plan, num_threads).await,
    }
}

/// Writes a CSV/TSV output from the sources
async fn write_file<I>(
    plan: OutputPartitionPlan,
    num_threads: usize,
    sources: I,
) -> Result<(), io::Error>
where
    I: Iterator<Item: Source> + 'static,
{
    // Since generate_in_chunks already buffers, there is no need to buffer
    // again (aka don't use BufWriter here)
    match plan.output_location() {
        OutputLocation::Stdout => {
            let sink = WriterSink::new(io::stdout());
            generate_in_chunks(sink, sources, num_threads).await
        }
        OutputLocation::File(path) => {
            let file = std::fs::File::create(path)?;
            let sink = WriterSink::new(file);
            generate_in_chunks(sink, sources, num_threads).await
        }
    }
}

/// Generates an output parquet file from the sources
async fn write_parquet<I>(
    plan: OutputPartitionPlan,
    num_threads: usize,
    sources: I,
) -> Result<(), io::Error>
where
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    match plan.output_location() {
        OutputLocation::Stdout => {
            let writer = BufWriter::with_capacity(32 * 1024 * 1024, io::stdout()); // 32MB buffer
            generate_parquet(writer, sources, num_threads, plan.parquet_compression()).await
        }
        OutputLocation::File(path) => {
            let file = std::fs::File::create(path)?;
            let writer = BufWriter::with_capacity(32 * 1024 * 1024, file); // 32MB buffer
            generate_parquet(writer, sources, num_threads, plan.parquet_compression()).await
        }
    }
}

/// macro to create a function for generating a part of a particular able
///
/// Arguments:
/// $FUN_NAME: name of the function to create
/// $GENERATOR: The generator type to use
/// $TBL_SOURCE: The [`Source`] type to use for TBL format
/// $CSV_SOURCE: The [`Source`] type to use for CSV format
/// $PARQUET_SOURCE: The [`RecordBatchIterator`] type to use for Parquet format
macro_rules! define_run {
    ($FUN_NAME:ident, $GENERATOR:ident, $TBL_SOURCE:ty, $CSV_SOURCE:ty, $PARQUET_SOURCE:ty) => {
        async fn $FUN_NAME(plan: OutputPartitionPlan, num_threads: usize) -> io::Result<()> {
            let scale_factor = plan.scale_factor();
            info!("Writing {plan} using {num_threads} threads");
            debug!("Plan: {plan:?}");

            // Create an interator of generators that will create the actual data
            let gens = plan
                .generation_plan()
                .clone()
                .into_iter()
                .map(move |(part, num_parts)| $GENERATOR::new(scale_factor, part, num_parts));

            // Dispach to the appropriate output format
            match plan.output_format() {
                OutputFormat::Tbl => {
                    write_file(plan, num_threads, gens.map(<$TBL_SOURCE>::new)).await
                }
                OutputFormat::Csv => {
                    write_file(plan, num_threads, gens.map(<$CSV_SOURCE>::new)).await
                }
                OutputFormat::Parquet => {
                    write_parquet(plan, num_threads, gens.map(<$PARQUET_SOURCE>::new)).await
                }
            }
        }
    };
}

define_run!(
    run_lineitem_plan,
    LineItemGenerator,
    LineItemTblSource,
    LineItemCsvSource,
    LineItemArrow
);

define_run!(
    run_nation_plan,
    NationGenerator,
    NationTblSource,
    NationCsvSource,
    NationArrow
);

define_run!(
    run_region_plan,
    RegionGenerator,
    RegionTblSource,
    RegionCsvSource,
    RegionArrow
);

define_run!(
    run_part_plan,
    PartGenerator,
    PartTblSource,
    PartCsvSource,
    PartArrow
);

define_run!(
    run_supplier_plan,
    SupplierGenerator,
    SupplierTblSource,
    SupplierCsvSource,
    SupplierArrow
);
define_run!(
    run_partsupp_plan,
    PartSuppGenerator,
    PartSuppTblSource,
    PartSuppCsvSource,
    PartSuppArrow
);

define_run!(
    run_customer_plan,
    CustomerGenerator,
    CustomerTblSource,
    CustomerCsvSource,
    CustomerArrow
);

define_run!(
    run_orders_plan,
    OrderGenerator,
    OrderTblSource,
    OrderCsvSource,
    OrderArrow
);
