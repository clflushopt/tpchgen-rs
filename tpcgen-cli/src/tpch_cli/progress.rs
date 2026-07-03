//! Progress tracking for table generation.
//!
//! # Overview
//!
//! [`ProgressTracker`] is a small, dyn-compatible trait that receives
//! generation events from [`crate::tpch_cli::runner::PlanRunner`]. The runner calls:
//!
//! 1. [`ProgressTracker::register`] once per table, before any worker
//!    starts, with the total number of output units the table will produce
//!    (chunks for TBL/CSV, row groups for Parquet).
//! 2. [`ProgressTracker::increment`] after output units are written.
//!    Multiple table-generation tasks may call it concurrently, so impls
//!    must be `Send + Sync` and `increment` itself should be lightweight.
//! 3. [`ProgressTracker::finish`] once on the success path when the
//!    runner exits. Implementations should use `finish` for normal
//!    success cleanup and `Drop` only as an error or panic fallback.
//!
//! `register` and `finish` are invoked serially by the runner and may
//! do bookkeeping or I/O; `increment` may run concurrently while output
//! is being written.
//!
//! Implementations must not panic and must not propagate I/O errors —
//! progress reporting is best-effort and must never affect the data
//! path.
//!
//! # Default implementation
//!
//! When the `indicatif-progress` feature is enabled (on by default), the
//! crate provides an `IndicatifProgress` implementation, which renders
//! one progress bar per table on stderr using the `indicatif` crate.
//! Library users who do not want to pull in `indicatif` can enable the
//! `progress` feature directly and supply their own [`ProgressTracker`]
//! implementation.
//!
//! # Example: a custom logging tracker
//!
//! ```
//! # #[cfg(feature = "progress")]
//! # {
//! use std::sync::atomic::{AtomicU64, Ordering};
//! use tpcgen_cli::tpch_cli::progress::ProgressTracker;
//! use tpcgen_cli::tpch_cli::Table;
//!
//! #[derive(Debug)]
//! struct LoggingTracker {
//!     written: AtomicU64,
//! }
//!
//! impl ProgressTracker for LoggingTracker {
//!     fn register(&self, table: Table, total: u64) {
//!         eprintln!("plan: {table:?} -> {total} output units");
//!     }
//!     fn increment(&self, _table: Table, units: u64) {
//!         self.written.fetch_add(units, Ordering::Relaxed);
//!     }
//!     fn finish(&self) {
//!         eprintln!("done: {} output units", self.written.load(Ordering::Relaxed));
//!     }
//! }
//! # }
//! ```

use crate::tpch_cli::output_plan::OutputPlan;
use crate::tpch_cli::Table;
#[cfg(feature = "progress")]
use std::collections::BTreeMap;
#[cfg(feature = "progress")]
use std::fmt;
#[cfg(feature = "progress")]
use std::sync::Arc;

/// Receives generation-progress events for one
/// [`PlanRunner`](crate::tpch_cli::runner::PlanRunner) invocation.
///
/// See the [module-level documentation](self) for the call-order
/// contract. Trackers are passed to the runner as an
/// [`std::sync::Arc`] and shared across concurrent generation tasks, so
/// they must be `Send + Sync`.
/// They must also be `Debug` so containing types can derive `Debug`.
#[cfg(feature = "progress")]
pub trait ProgressTracker: Send + Sync + fmt::Debug {
    /// Pre-register a table with its total expected output-unit count.
    ///
    /// Called once per table before any worker starts. Implementations
    /// that need to know totals up front (e.g. to render a progress bar
    /// or compute an ETA) should override this; the default does
    /// nothing.
    fn register(&self, _table: Table, _total_units: u64) {}

    /// Advance the counter for `table` by `units` output units.
    ///
    /// Called after generated output units are written. Multiple
    /// table-generation tasks may call this concurrently, so implementations
    /// should be lightweight and must never panic.
    fn increment(&self, table: Table, units: u64);

    /// Called once after the last [`Self::increment`] on the success
    /// path. Implementations should use this for normal success cleanup
    /// and `Drop` only as an error or panic fallback. The default does
    /// nothing.
    fn finish(&self) {}
}

/// Progress handle for one [`PlanRunner::run`](crate::tpch_cli::runner::PlanRunner::run)
/// invocation.
///
/// Owns run-level progress lifecycle: registering totals, accounting for
/// skipped outputs, and finishing the tracker.
#[derive(Debug, Clone, Default)]
#[cfg(feature = "progress")]
pub(crate) struct RunProgress {
    tracker: Option<Arc<dyn ProgressTracker>>,
}

#[cfg(feature = "progress")]
impl RunProgress {
    pub(crate) fn with_tracker(tracker: Arc<dyn ProgressTracker>) -> Self {
        Self {
            tracker: Some(tracker),
        }
    }

    pub(crate) fn register_totals(&self, plans: &[OutputPlan]) {
        if let Some(tracker) = self.tracker.as_ref() {
            let mut totals: BTreeMap<Table, u64> = BTreeMap::new();
            for plan in plans {
                *totals.entry(plan.table()).or_insert(0) += plan.chunk_count() as u64;
            }
            for (table, total) in totals {
                tracker.register(table, total);
            }
        }
    }

    pub(crate) fn increment_for_existing(&self, plan: &OutputPlan) {
        if let Some(tracker) = self.tracker.as_ref() {
            tracker.increment(plan.table(), plan.chunk_count() as u64);
        }
    }

    pub(crate) fn for_table(&self, table: Table) -> TableProgress {
        TableProgress::for_table(self.tracker.clone(), table)
    }

    pub(crate) fn finish(self) {
        if let Some(tracker) = self.tracker {
            tracker.finish();
        }
    }
}

/// No-op run progress handle used when progress support is disabled.
#[derive(Debug, Clone, Default)]
#[cfg(not(feature = "progress"))]
pub(crate) struct RunProgress;

#[cfg(not(feature = "progress"))]
impl RunProgress {
    pub(crate) fn register_totals(&self, _plans: &[OutputPlan]) {}

    pub(crate) fn increment_for_existing(&self, _plan: &OutputPlan) {}

    pub(crate) fn for_table(&self, _table: Table) -> TableProgress {
        TableProgress
    }

    pub(crate) fn finish(self) {}
}

/// Progress handle for one table output stream.
///
/// Used by format writers to report each successfully written output unit
/// without knowing whether progress tracking is enabled.
#[derive(Clone, Default)]
#[cfg(feature = "progress")]
pub(crate) struct TableProgress {
    tracker: Option<(Arc<dyn ProgressTracker>, Table)>,
}

#[cfg(feature = "progress")]
impl TableProgress {
    pub(crate) fn for_table(progress: Option<Arc<dyn ProgressTracker>>, table: Table) -> Self {
        Self {
            tracker: progress.map(|progress| (progress, table)),
        }
    }

    pub(crate) fn increment_output_unit(&self) {
        if let Some((progress, table)) = self.tracker.as_ref() {
            progress.increment(*table, 1);
        }
    }
}

/// No-op table progress handle used when progress support is disabled.
#[derive(Clone, Default)]
#[cfg(not(feature = "progress"))]
pub(crate) struct TableProgress;

#[cfg(not(feature = "progress"))]
impl TableProgress {
    pub(crate) fn increment_output_unit(&self) {}
}

#[cfg(feature = "indicatif-progress")]
pub use indicatif_impl::IndicatifProgress;

#[cfg(feature = "indicatif-progress")]
mod indicatif_impl {
    use super::ProgressTracker;
    use crate::tpch_cli::Table;
    use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
    use std::collections::BTreeMap;
    use std::io::{self, Write};
    use std::sync::{OnceLock, RwLock};

    /// Default [`ProgressTracker`] implementation backed by
    /// [`indicatif::MultiProgress`].
    ///
    /// Renders one bar per table on stderr. Bars are pre-allocated in
    /// [`ProgressTracker::register`] and are looked up by [`Table`] on
    /// each [`ProgressTracker::increment`] call. Lookup uses a `RwLock`
    /// read on the increment path; this is uncontended after the serial
    /// `register` phase completes.
    #[derive(Debug)]
    pub struct IndicatifProgress {
        multi: MultiProgress,
        tables: RwLock<BTreeMap<Table, ProgressBar>>,
    }

    impl IndicatifProgress {
        /// Construct an empty tracker. Tables are added via
        /// [`ProgressTracker::register`].
        pub fn new() -> Self {
            Self {
                multi: MultiProgress::new(),
                tables: RwLock::new(BTreeMap::new()),
            }
        }

        /// Return a writer that coordinates stderr log writes with progress
        /// bar redraws.
        pub fn log_writer(&self) -> Box<dyn io::Write + Send + 'static> {
            Box::new(IndicatifLogWriter {
                multi: self.multi.clone(),
            })
        }
    }

    impl Default for IndicatifProgress {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProgressTracker for IndicatifProgress {
        fn register(&self, table: Table, total_units: u64) {
            let Ok(mut tables) = self.tables.write() else {
                return;
            };

            let pb = self.multi.add(ProgressBar::new(total_units));
            pb.set_style(bar_style().clone());
            pb.set_message(table.to_string());
            let pb = pb.with_finish(ProgressFinish::AndLeave);
            // Write-lock is only contended during the register phase, which
            // happens serially before any worker task starts.
            tables.insert(table, pb);
        }

        fn increment(&self, table: Table, units: u64) {
            // Minimize the read-lock scope so concurrent `increment` callers
            // don't serialize on it. Cloning the bar is a cheap `Arc` bump,
            // and `ProgressBar::inc` is internally thread-safe.
            let bar = {
                let Ok(tables) = self.tables.read() else {
                    return;
                };
                tables.get(&table).cloned()
            };
            if let Some(bar) = bar {
                bar.inc(units);
            }
        }

        fn finish(&self) {
            let bars = {
                let Ok(tables) = self.tables.read() else {
                    return;
                };
                tables.values().cloned().collect::<Vec<_>>()
            };
            for bar in bars {
                bar.finish_using_style();
            }
        }
    }

    fn bar_style() -> &'static ProgressStyle {
        static STYLE: OnceLock<ProgressStyle> = OnceLock::new();
        STYLE.get_or_init(|| {
            ProgressStyle::default_bar()
                .template("{msg:10} [{bar:28}]   Progress: {percent:>3}%")
                .expect("static progress bar template is valid")
                .progress_chars("█▓░")
        })
    }

    struct IndicatifLogWriter {
        multi: MultiProgress,
    }

    impl Write for IndicatifLogWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.multi.suspend(|| {
                let mut stderr = io::stderr().lock();
                stderr.write(buf)
            })
        }

        fn flush(&mut self) -> io::Result<()> {
            self.multi.suspend(|| {
                let mut stderr = io::stderr().lock();
                stderr.flush()
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn registers_and_increments() {
            let t = IndicatifProgress::new();
            t.register(Table::Lineitem, 60);
            t.register(Table::Orders, 15);
            t.increment(Table::Lineitem, 1);
            t.increment(Table::Orders, 5);

            let tables = t.tables.read().unwrap();
            assert_eq!(tables[&Table::Lineitem].position(), 1);
            assert_eq!(tables[&Table::Orders].position(), 5);
        }

        #[test]
        fn reaches_total() {
            let t = IndicatifProgress::new();
            t.register(Table::Orders, 5);
            for _ in 0..5 {
                t.increment(Table::Orders, 1);
            }
            assert_eq!(t.tables.read().unwrap()[&Table::Orders].position(), 5);
        }

        #[test]
        fn unknown_table_is_no_op() {
            // Incrementing a table not registered must not panic.
            let t = IndicatifProgress::new();
            t.register(Table::Orders, 1);
            t.increment(Table::Lineitem, 1);
            assert_eq!(t.tables.read().unwrap()[&Table::Orders].position(), 0);
        }

        #[test]
        fn finish_marks_registered_bars_finished() {
            let t = IndicatifProgress::new();
            t.register(Table::Orders, 2);
            t.increment(Table::Orders, 2);
            t.finish();

            assert!(t.tables.read().unwrap()[&Table::Orders].is_finished());
        }
    }
}

#[cfg(all(test, feature = "progress"))]
mod tests {
    use super::*;
    use crate::tpch_cli::Table;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    };

    /// Mock implementation that records every event. Demonstrates that
    /// the trait is dyn-compatible and usable from external code without
    /// pulling in `indicatif`.
    #[derive(Debug, Default)]
    struct MockTracker {
        registered: Mutex<Vec<(Table, u64)>>,
        total_increments: AtomicU64,
        finished: AtomicU64,
    }

    impl ProgressTracker for MockTracker {
        fn register(&self, table: Table, total_units: u64) {
            self.registered.lock().unwrap().push((table, total_units));
        }
        fn increment(&self, _table: Table, units: u64) {
            self.total_increments.fetch_add(units, Ordering::Relaxed);
        }
        fn finish(&self) {
            self.finished.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn mock_tracker_works_through_arc_dyn() {
        let mock = Arc::new(MockTracker::default());
        let dynamic: Arc<dyn ProgressTracker> = mock.clone();
        dynamic.register(Table::Lineitem, 10);
        dynamic.register(Table::Orders, 4);
        dynamic.increment(Table::Lineitem, 3);
        dynamic.increment(Table::Orders, 1);
        dynamic.finish();

        assert_eq!(
            *mock.registered.lock().unwrap(),
            vec![(Table::Lineitem, 10), (Table::Orders, 4)]
        );
        assert_eq!(mock.total_increments.load(Ordering::Relaxed), 4);
        assert_eq!(mock.finished.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn default_register_and_finish_are_noops() {
        // An impl that only overrides `increment` should compile and run.
        #[derive(Debug)]
        struct Minimal(AtomicU64);
        impl ProgressTracker for Minimal {
            fn increment(&self, _t: Table, c: u64) {
                self.0.fetch_add(c, Ordering::Relaxed);
            }
        }
        let m = Minimal(AtomicU64::new(0));
        m.register(Table::Region, 99); // no-op default
        m.increment(Table::Region, 7);
        m.finish(); // no-op default
        assert_eq!(m.0.load(Ordering::Relaxed), 7);
    }
}
