//! `tpch updates`: the TPC-H refresh function update sets (`dbgen -U`).

use crate::args::{
    check_scale_factor, default_num_threads, parse_scale_factor, TPCH_SCALE_FACTORS,
};
use crate::generate::{Source, TextOutput};
use crate::logging::configure_logging;
use crate::output_location::OutputLocation;
use crate::progress::no_op_progress_tracker;
use crate::tpch_cli::plan::tbl_chunk_count;
use crate::tpch_cli::tbl::{DeleteTblSource, LineItemTblSource, OrderTblSource};
use crate::tpch_cli::Table;
use crate::worker_queue::WorkerQueue;
use std::io;
use std::path::PathBuf;
use tpchgen::generators::{DeleteGenerator, LineItemGenerator, OrderGenerator, UpdateSet};

/// Options of `tpch updates`
#[derive(clap::Args)]
#[command(next_help_heading = "Update Options")]
pub(super) struct UpdatesArgs {
    /// Scale factor of the data the update sets apply to
    #[arg(short, long, default_value_t = 1., value_parser = parse_scale_factor)]
    scale_factor: f64,

    /// Number of update sets to generate (`dbgen -U`)
    #[arg(
        long,
        default_value_t = 1,
        value_name = "N",
        value_parser = clap::value_parser!(i32).range(1..)
    )]
    sets: i32,

    /// Output directory for generated files
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Overwrite output files that already exist.
    #[arg(long, default_value_t = false)]
    overwrite: bool,

    /// Number of threads to use [default: number of CPUs]
    #[arg(
        short,
        long,
        default_value_t = default_num_threads(),
        hide_default_value = true,
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..)
    )]
    num_threads: usize,

    /// Log at info level (overrides RUST_LOG)
    #[arg(short, long, default_value_t = false, conflicts_with = "quiet")]
    verbose: bool,

    /// Only log errors
    #[arg(short, long, default_value_t = false, conflicts_with = "verbose")]
    quiet: bool,
}

impl UpdatesArgs {
    pub(super) async fn run(self) -> io::Result<()> {
        configure_logging(self.verbose, self.quiet, None);
        check_scale_factor("TPC-H", self.scale_factor, TPCH_SCALE_FACTORS);
        let sf = self.scale_factor;
        // Like dbgen, write only the (empty) delete files if the update sets
        // are empty
        let row_count = UpdateSet::row_count(sf);
        if row_count == 0 {
            log::warn!("Update sets are empty below scale factor 0.01 (scale factor {sf})");
        } else {
            // Like dbgen, allow more sets than there are base orders to delete
            let max_sets = OrderGenerator::calculate_row_count(sf, 1, 1) / row_count;
            if self.sets as i64 > max_sets {
                log::warn!(
                    "Update sets after set {max_sets} delete orders that are not in the base data \
                     and may insert duplicate order keys (requested {} sets)",
                    self.sets
                );
            }
        }
        let order_chunks = tbl_chunk_count(Table::Orders, row_count);
        // there are on average 4 line items per order
        let lineitem_chunks = tbl_chunk_count(Table::Lineitem, 4 * row_count);
        let location = OutputLocation::new(false, self.output_dir, self.overwrite);
        location.create_dir_all()?;

        // The files are independent, so generate them in parallel, the
        // largest first: the line items of all sets, then the orders, then
        // the deletes
        let mut worker_queue = WorkerQueue::new(self.num_threads);
        if row_count > 0 {
            for set in 1..=self.sets {
                let sources = (1..=lineitem_chunks).map(move |part| {
                    LineItemTblSource::new(LineItemGenerator::update_set(
                        sf,
                        set,
                        part,
                        lineitem_chunks,
                    ))
                });
                let file = location.join(format!("lineitem.tbl.u{set}"));
                schedule(&mut worker_queue, file, lineitem_chunks, sources).await?;
            }
            for set in 1..=self.sets {
                let sources = (1..=order_chunks).map(move |part| {
                    OrderTblSource::new(OrderGenerator::update_set(sf, set, part, order_chunks))
                });
                let file = location.join(format!("orders.tbl.u{set}"));
                schedule(&mut worker_queue, file, order_chunks, sources).await?;
            }
        }
        for set in 1..=self.sets {
            let sources = std::iter::once(DeleteTblSource::new(DeleteGenerator::new(sf, set)));
            let file = location.join(format!("delete.{set}"));
            schedule(&mut worker_queue, file, 1, sources).await?;
        }
        worker_queue.join_all().await
    }
}

/// Schedule writing `chunk_count` `sources` in TBL format to `location`
async fn schedule<I>(
    worker_queue: &mut WorkerQueue,
    location: OutputLocation,
    chunk_count: i32,
    sources: I,
) -> io::Result<()>
where
    I: Iterator<Item: Source + 'static> + Send + 'static,
{
    worker_queue
        .schedule(chunk_count as usize, move |num_threads| async move {
            let written = location
                .write(TextOutput {
                    sources,
                    num_threads,
                    progress: no_op_progress_tracker().register("", chunk_count as u64),
                })
                .await?;
            if written {
                log::info!("Generated {location}");
            }
            Ok(num_threads)
        })
        .await
}
