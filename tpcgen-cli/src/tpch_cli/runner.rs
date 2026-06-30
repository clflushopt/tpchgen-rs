//! [`PlanRunner`] for running [`OutputPlan`]s.

use crate::tpch_cli::csv::*;
use crate::tpch_cli::generate::generate_in_chunks_with_progress;
use crate::tpch_cli::generate::Source;
use crate::tpch_cli::output_plan::{OutputLocation, OutputPlan};
use crate::tpch_cli::parquet::generate_parquet_with_progress;
#[cfg(feature = "progress")]
use crate::tpch_cli::progress::ProgressTracker;
use crate::tpch_cli::progress::RunProgress;
use crate::tpch_cli::tbl::*;
use crate::tpch_cli::tbl::{LineItemTblSource, NationTblSource, RegionTblSource};
use crate::tpch_cli::{OutputFormat, Table, WriterSink};
use log::{debug, info};
use std::io;
use std::io::BufWriter;
#[cfg(feature = "progress")]
use std::sync::Arc;
use tokio::task::{JoinError, JoinSet};
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSuppGenerator, RegionGenerator, SupplierGenerator,
};
use tpchgen_arrow::{
    CustomerArrow, LineItemArrow, NationArrow, OrderArrow, PartArrow, PartSuppArrow,
    RecordBatchIterator, RegionArrow, SupplierArrow,
};

/// Runs multiple [`OutputPlan`]s in parallel, managing the number of threads
/// used to run them.
#[derive(Debug)]
pub struct PlanRunner {
    plans: Vec<OutputPlan>,
    num_threads: usize,
    progress: RunProgress,
}

impl PlanRunner {
    /// Create a new [`PlanRunner`] with the given plans and number of threads.
    /// Progress reporting is disabled by default.
    pub fn new(plans: Vec<OutputPlan>, num_threads: usize) -> Self {
        Self {
            plans,
            num_threads,
            progress: RunProgress::default(),
        }
    }

    /// Attach a [`ProgressTracker`].
    ///
    /// The runner pre-registers each table's output-unit total with the
    /// tracker before scheduling, calls [`ProgressTracker::increment`]
    /// after output units are written, and calls [`ProgressTracker::finish`]
    /// once on the success path. Implementations needing cleanup on the
    /// error or panic path should use `Drop` as a fallback.
    #[cfg(feature = "progress")]
    pub fn with_progress_tracker(mut self, tracker: Arc<dyn ProgressTracker>) -> Self {
        self.progress = RunProgress::with_tracker(tracker);
        self
    }

    /// Run all the plans in the runner.
    pub async fn run(self) -> Result<(), io::Error> {
        debug!(
            "Running {} plans with {} threads...",
            self.plans.len(),
            self.num_threads
        );
        let Self {
            mut plans,
            num_threads,
            progress,
        } = self;

        // Sort the plans by the number of parts so the largest are first
        plans.sort_unstable_by(|a, b| {
            let a_cnt = a.chunk_count();
            let b_cnt = b.chunk_count();
            a_cnt.cmp(&b_cnt)
        });

        // Pre-register per-table output-unit totals so trackers can size their
        // bars before the first `increment`.
        progress.register_totals(&plans);

        // Do the actual work in parallel, using a worker queue
        let mut worker_queue = WorkerQueue::new(num_threads, progress.clone());
        while let Some(plan) = plans.pop() {
            worker_queue.schedule_plan(plan).await?;
        }
        worker_queue.join_all().await?;
        progress.finish();
        Ok(())
    }
}

/// Manages worker tasks, limiting the number of total outstanding threads
/// to some fixed number
///
/// The runner executes each plan with a number of threads equal to the
/// number of parts in the plan, but no more than the total number of
/// threads specified when creating the runner. If a plan does not need all
/// the threads, the remaining threads are used to run other plans.
///
/// This is important to keep all cores busy for smaller tables that may not
/// have sufficient parts to keep all threads busy (see [`GenerationPlan`]
/// for more details), but not schedule more tasks than we have threads for.
///
/// Scheduling too many tasks requires more memory and leads to context
/// switching overhead, which can slow down the generation process.
///
/// [`GenerationPlan`]: crate::tpch_cli::plan::GenerationPlan
struct WorkerQueue {
    join_set: JoinSet<io::Result<usize>>,
    /// Current number of threads available to commit
    available_threads: usize,
    progress: RunProgress,
}

impl WorkerQueue {
    pub fn new(max_threads: usize, progress: RunProgress) -> Self {
        assert!(max_threads > 0);
        Self {
            join_set: JoinSet::new(),
            available_threads: max_threads,
            progress,
        }
    }

    /// Spawns a task to run the plan with as many threads as possible
    /// without exceeding the maximum number of threads.
    ///
    /// If there are no threads available, it will wait for one to finish
    /// before spawning the new task.
    ///
    /// Note this algorithm does not guarantee that all threads are always busy,
    /// but it should be good enough for most cases. For best thread utilization
    /// spawn the largest plans first.
    pub async fn schedule_plan(&mut self, plan: OutputPlan) -> io::Result<()> {
        debug!("scheduling plan {plan}");
        loop {
            if self.available_threads == 0 {
                debug!("no threads left, wait for one to finish");
                let Some(result) = self.join_set.join_next().await else {
                    return Err(io::Error::other(
                        "Internal Error No more tasks to wait for, but had no threads",
                    ));
                };
                self.available_threads += task_result(result)?;
                continue; // look for threads again
            }

            // Check for any other jobs done so we can reuse their threads
            if let Some(result) = self.join_set.try_join_next() {
                self.available_threads += task_result(result)?;
                continue;
            }

            debug_assert!(
                self.available_threads > 0,
                "should have at least one thread to continue"
            );

            // figure out how many threads to allocate to this plan. Each plan
            // can use up to `part_count` threads.
            let chunk_count = plan.chunk_count();

            let num_plan_threads = self.available_threads.min(chunk_count);

            // run the plan in a separate task, which returns the number of threads it used
            debug!("Spawning plan {plan} with {num_plan_threads} threads");

            let progress = self.progress.clone();
            self.join_set
                .spawn(async move { run_plan(plan, num_plan_threads, progress).await });
            self.available_threads -= num_plan_threads;
            return Ok(());
        }
    }

    // Wait for all tasks to finish
    pub async fn join_all(mut self) -> io::Result<()> {
        debug!("Waiting for tasks to finish...");
        while let Some(result) = self.join_set.join_next().await {
            task_result(result)?;
        }
        debug!("Tasks finished.");
        Ok(())
    }
}

/// unwraps the result of a task and converts it to an `io::Result<T>`.
fn task_result<T>(result: Result<io::Result<T>, JoinError>) -> io::Result<T> {
    result.map_err(|e| io::Error::other(format!("Task Panic: {e}")))?
}

/// Run a single [`OutputPlan`]
async fn run_plan(
    plan: OutputPlan,
    num_threads: usize,
    progress: RunProgress,
) -> io::Result<usize> {
    match plan.table() {
        Table::Nation => run_nation_plan(plan, num_threads, progress).await,
        Table::Region => run_region_plan(plan, num_threads, progress).await,
        Table::Part => run_part_plan(plan, num_threads, progress).await,
        Table::Supplier => run_supplier_plan(plan, num_threads, progress).await,
        Table::Partsupp => run_partsupp_plan(plan, num_threads, progress).await,
        Table::Customer => run_customer_plan(plan, num_threads, progress).await,
        Table::Orders => run_orders_plan(plan, num_threads, progress).await,
        Table::Lineitem => run_lineitem_plan(plan, num_threads, progress).await,
    }
}

/// If `path` already exists, log a warning, advance progress by the full
/// output-unit count for this plan, and return `true` so the caller can skip
/// generation. Returns `false` otherwise.
fn maybe_skip_existing(path: &std::path::Path, plan: &OutputPlan, progress: &RunProgress) -> bool {
    if !path.exists() {
        return false;
    }
    log::warn!("{} already exists, skipping generation", path.display());
    progress.increment_for_existing(plan);
    true
}

/// Writes a CSV/TSV output from the sources
async fn write_file<I>(
    plan: OutputPlan,
    num_threads: usize,
    sources: I,
    progress: RunProgress,
) -> Result<(), io::Error>
where
    I: Iterator<Item: Source> + 'static,
{
    let table = plan.table();
    let table_progress = progress.for_table(table);
    // Since generate_in_chunks already buffers, there is no need to buffer
    // again (aka don't use BufWriter here)
    match plan.output_location() {
        OutputLocation::Stdout => {
            let sink = WriterSink::new(io::stdout());
            generate_in_chunks_with_progress(sink, sources, num_threads, table_progress).await
        }
        OutputLocation::File(path) => {
            if maybe_skip_existing(path, &plan, &progress) {
                return Ok(());
            }
            // write to a temp file and then rename to avoid partial files
            let temp_path = path.with_extension("inprogress");
            let file = std::fs::File::create(&temp_path).map_err(|err| {
                io::Error::other(format!("Failed to create {temp_path:?}: {err}"))
            })?;
            let sink = WriterSink::new(file);
            generate_in_chunks_with_progress(sink, sources, num_threads, table_progress).await?;
            // rename the temp file to the final path
            std::fs::rename(&temp_path, path).map_err(|e| {
                io::Error::other(format!(
                    "Failed to rename {temp_path:?} to {path:?} file: {e}"
                ))
            })?;
            Ok(())
        }
    }
}

/// Generates an output parquet file from the sources
async fn write_parquet<I>(
    plan: OutputPlan,
    num_threads: usize,
    sources: I,
    progress: RunProgress,
) -> Result<(), io::Error>
where
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    let table = plan.table();
    let table_progress = progress.for_table(table);
    match plan.output_location() {
        OutputLocation::Stdout => {
            let writer = BufWriter::with_capacity(32 * 1024 * 1024, io::stdout()); // 32MB buffer
            generate_parquet_with_progress(
                writer,
                sources,
                num_threads,
                plan.parquet_compression(),
                table_progress,
            )
            .await
        }
        OutputLocation::File(path) => {
            if maybe_skip_existing(path, &plan, &progress) {
                return Ok(());
            }
            // write to a temp file and then rename to avoid partial files
            let temp_path = path.with_extension("inprogress");
            let file = std::fs::File::create(&temp_path).map_err(|err| {
                io::Error::other(format!("Failed to create {temp_path:?}: {err}"))
            })?;
            let writer = BufWriter::with_capacity(32 * 1024 * 1024, file); // 32MB buffer
            generate_parquet_with_progress(
                writer,
                sources,
                num_threads,
                plan.parquet_compression(),
                table_progress,
            )
            .await?;
            // rename the temp file to the final path
            std::fs::rename(&temp_path, path).map_err(|e| {
                io::Error::other(format!(
                    "Failed to rename {temp_path:?} to {path:?} file: {e}"
                ))
            })?;
            Ok(())
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
        async fn $FUN_NAME(
            plan: OutputPlan,
            num_threads: usize,
            progress: RunProgress,
        ) -> io::Result<usize> {
            use crate::tpch_cli::GenerationPlan;
            let scale_factor = plan.scale_factor();
            info!("Writing {plan} using {num_threads} threads");

            /// These interior functions are used to tell the compiler that the lifetime is 'static
            /// (when these were closures, the compiler could not figure out the lifetime) and
            /// resulted in errors like this:
            ///          let _ = join_set.spawn(async move {
            ///                 |  _____________________^
            ///              96 | |                 run_plan(plan, num_plan_threads).await
            ///              97 | |             });
            ///                 | |______________^ implementation of `FnOnce` is not general enough
            fn tbl_sources(
                generation_plan: &GenerationPlan,
                scale_factor: f64,
            ) -> impl Iterator<Item: Source> + 'static {
                generation_plan
                    .clone()
                    .into_iter()
                    .map(move |(part, num_parts)| $GENERATOR::new(scale_factor, part, num_parts))
                    .map(<$TBL_SOURCE>::new)
            }

            fn csv_sources(
                generation_plan: &GenerationPlan,
                scale_factor: f64,
                delimiter: char,
            ) -> impl Iterator<Item: Source> + 'static {
                generation_plan
                    .clone()
                    .into_iter()
                    .map(move |(part, num_parts)| $GENERATOR::new(scale_factor, part, num_parts))
                    .map(move |gen| <$CSV_SOURCE>::new(gen, delimiter))
            }

            fn parquet_sources(
                generation_plan: &GenerationPlan,
                scale_factor: f64,
            ) -> impl Iterator<Item: RecordBatchIterator> + 'static {
                generation_plan
                    .clone()
                    .into_iter()
                    .map(move |(part, num_parts)| $GENERATOR::new(scale_factor, part, num_parts))
                    .map(<$PARQUET_SOURCE>::new)
            }

            // Dispatch to the appropriate output format
            match plan.output_format() {
                OutputFormat::Tbl => {
                    let gens = tbl_sources(plan.generation_plan(), scale_factor);
                    write_file(plan, num_threads, gens, progress).await?
                }
                OutputFormat::Csv => {
                    let delimiter = plan.csv_delimiter();
                    let gens = csv_sources(plan.generation_plan(), scale_factor, delimiter);
                    write_file(plan, num_threads, gens, progress).await?
                }
                OutputFormat::Parquet => {
                    let gens = parquet_sources(plan.generation_plan(), scale_factor);
                    write_parquet(plan, num_threads, gens, progress).await?
                }
            };
            Ok(num_threads)
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

#[cfg(all(test, feature = "progress"))]
mod tests {
    use super::*;
    use crate::tpch_cli::progress::ProgressTracker;
    use crate::tpch_cli::{Compression, GenerationPlan, DEFAULT_PARQUET_ROW_GROUP_BYTES};
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    #[derive(Debug)]
    struct CountingProgress {
        increments: AtomicU64,
    }

    impl ProgressTracker for CountingProgress {
        fn increment(&self, _table: Table, units: u64) {
            self.increments.fetch_add(units, Ordering::Relaxed);
        }
    }

    #[test]
    fn skip_existing_advances_progress_by_full_plan() {
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("lineitem.tbl");
        std::fs::write(&output_path, b"already here").unwrap();

        let generation_plan = GenerationPlan::try_new(
            Table::Lineitem,
            OutputFormat::Tbl,
            1.0,
            Some(1),
            Some(4),
            DEFAULT_PARQUET_ROW_GROUP_BYTES,
        )
        .unwrap();
        let plan = OutputPlan::new(
            Table::Lineitem,
            1.0,
            OutputFormat::Tbl,
            Compression::SNAPPY,
            OutputLocation::File(output_path.clone()),
            generation_plan,
            ',',
        );
        let expected_units = plan.chunk_count() as u64;
        assert!(expected_units > 1);

        let tracker = Arc::new(CountingProgress {
            increments: AtomicU64::new(0),
        });
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let progress = RunProgress::with_tracker(progress);

        assert!(maybe_skip_existing(&output_path, &plan, &progress));
        assert_eq!(tracker.increments.load(Ordering::Relaxed), expected_units);
    }
}
