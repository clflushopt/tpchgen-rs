//! Progress tracking for data generation.
//!
//! # Overview
//!
//! [`ProgressTracker`] is a small, dyn-compatible trait that owns run-level
//! progress lifecycle. Generation code uses it as follows:
//!
//! 1. [`ProgressTracker::register`] once per progress item, before work
//!    starts, with the total number of output units the item will produce
//!    (chunks for TBL/CSV, row groups for Parquet). Registration returns a
//!    [`ProgressHandle`] bound to that item.
//! 2. [`ProgressTracker::start`] once after all known progress items have
//!    been registered and before work starts. This hook is optional for
//!    paths that register items lazily.
//! 3. [`ProgressHandle::increment`] after output units are written.
//!    Multiple generation tasks may call handles concurrently, so their
//!    callbacks must be `Send + Sync` and lightweight.
//! 4. [`ProgressTracker::finish`] once on the success path when the
//!    generation run exits. Implementations should use `finish` for normal
//!    success cleanup and `Drop` only as an error or panic fallback.
//!
//! When invoked, `register`, `start`, and `finish` are serial and may do
//! bookkeeping or I/O; handles may be incremented concurrently while output
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
//! one progress bar per progress item on stderr using the `indicatif` crate.
//! Library users who do not want to pull in `indicatif` can disable default
//! features and still supply their own [`ProgressTracker`] implementation.
//! Without `indicatif-progress` and without a custom tracker, progress
//! reporting is a no-op.
//!
//! # Example: a custom logging tracker
//!
//! ```
//! use std::sync::atomic::{AtomicU64, Ordering};
//! use std::sync::Arc;
//! use tpcgen_cli::progress::{ProgressHandle, ProgressTracker};
//!
//! #[derive(Debug)]
//! struct LoggingTracker {
//!     written: AtomicU64,
//! }
//!
//! impl ProgressTracker for LoggingTracker {
//!     fn register(self: Arc<Self>, item: &str, total: u64) -> ProgressHandle {
//!         eprintln!("plan: {item} -> {total} output units");
//!         ProgressHandle::new(move |units| {
//!             self.written.fetch_add(units, Ordering::Relaxed);
//!         })
//!     }
//!     fn finish(&self) {
//!         eprintln!("done: {} output units", self.written.load(Ordering::Relaxed));
//!     }
//! }
//! ```

use std::fmt;
use std::sync::Arc;

/// Receives generation-progress events for one generation run.
///
/// See the [module-level documentation](self) for the call-order
/// contract. Trackers are held in a [`std::sync::Arc`] and shared across
/// concurrent generation tasks through the [`ProgressHandle`]s returned by
/// [`Self::register`], so they must be `Send + Sync`.
/// They must also be `Debug` so containing types can derive `Debug`.
pub trait ProgressTracker: Send + Sync + fmt::Debug {
    /// Pre-register a progress item with its total expected output-unit count.
    ///
    /// Called once per item before work starts. The `item` is a stable
    /// identifier, usually a table name. The returned handle is the only
    /// capability generation tasks need to advance that item.
    ///
    /// The [`Arc`] receiver lets implementations return a `'static` handle
    /// that shares tracker state with concurrent generation tasks.
    fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle;

    /// Optional hook called after all known progress items have been
    /// registered and before the first [`ProgressHandle::increment`].
    ///
    /// Implementations can use this to finalize setup that depends on the
    /// registered item set. The default does nothing.
    fn start(&self) {}

    /// Called once after the last [`ProgressHandle::increment`] on the success
    /// path. Implementations should use this for normal success cleanup
    /// and `Drop` only as an error or panic fallback. The default does
    /// nothing.
    fn finish(&self) {}
}

/// A cloneable handle for reporting progress for one registered item.
///
/// Handles are created by [`ProgressTracker::register`] and directly own the
/// item-specific increment behavior. Run-level lifecycle operations remain the
/// responsibility of the tracker owner.
#[derive(Clone)]
pub struct ProgressHandle {
    increment: Arc<dyn Fn(u64) + Send + Sync>,
}

impl ProgressHandle {
    /// Create a handle backed by `increment`.
    pub fn new<F>(increment: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        Self {
            increment: Arc::new(increment),
        }
    }

    /// Advance this item's counter by `units` output units.
    pub fn increment(&self, units: u64) {
        (self.increment)(units);
    }

    fn no_op() -> Self {
        Self::new(|_| {})
    }
}

impl fmt::Debug for ProgressHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressHandle").finish_non_exhaustive()
    }
}

/// Default tracker used when no progress backend is installed.
///
/// This keeps generation on the same always-reporting path; the no-op
/// implementation simply ignores every event.
#[derive(Debug, Default)]
struct NoOpProgressTracker;

impl ProgressTracker for NoOpProgressTracker {
    fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
        ProgressHandle::no_op()
    }
}

pub(crate) fn no_op_progress_tracker() -> Arc<dyn ProgressTracker> {
    Arc::new(NoOpProgressTracker)
}

#[cfg(feature = "indicatif-progress")]
pub use indicatif_impl::IndicatifProgress;

#[cfg(feature = "indicatif-progress")]
mod indicatif_impl {
    use super::{ProgressHandle, ProgressTracker};
    use indicatif::ProgressDrawTarget;
    use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

    const LABEL_WIDTH: usize = 22;
    const BAR_WIDTH: usize = 18;
    // 5 Hz redraws every 200 ms, keeping multi-bar updates responsive without repainting too often.
    const PROGRESS_REFRESH_HZ: u8 = 5;
    const PROGRESS_CHARS: &str = "=>-";

    /// Default [`ProgressTracker`] implementation backed by
    /// [`indicatif::MultiProgress`].
    ///
    /// Renders one compact progress bar per progress item on stderr.
    ///
    /// Items are added in [`ProgressTracker::register`], which returns a handle
    /// that advances its progress bar directly.
    #[derive(Debug)]
    pub struct IndicatifProgress {
        multi: MultiProgress,
        bars: Mutex<Vec<ProgressBar>>,
    }

    impl IndicatifProgress {
        /// Construct an empty tracker. Progress items are added via
        /// [`ProgressTracker::register`].
        pub fn new() -> Self {
            Self {
                multi: MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(
                    PROGRESS_REFRESH_HZ,
                )),
                bars: Mutex::new(Vec::new()),
            }
        }

        /// Return a writer that coordinates stderr log writes with progress
        /// bar redraws.
        pub fn log_writer(&self) -> Box<dyn io::Write + Send + 'static> {
            Box::new(IndicatifLogWriter {
                multi: self.multi.clone(),
            })
        }

        fn lock_bars(&self) -> MutexGuard<'_, Vec<ProgressBar>> {
            self.bars
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        }

        #[cfg(test)]
        fn hidden() -> Self {
            Self {
                multi: MultiProgress::with_draw_target(ProgressDrawTarget::hidden()),
                bars: Mutex::new(Vec::new()),
            }
        }
    }

    impl Default for IndicatifProgress {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProgressTracker for IndicatifProgress {
        fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle {
            // Indicatif treats zero-length items as complete.
            let bar_len = total_units.max(1);
            let pb = self.multi.add(
                ProgressBar::new(bar_len)
                    .with_style(bar_style())
                    .with_message(item.to_owned())
                    .with_finish(ProgressFinish::AndLeave),
            );
            self.lock_bars().push(pb.clone());
            ProgressHandle::new(move |units| pb.inc(units))
        }

        fn start(&self) {
            let bars = self.lock_bars().clone();

            // Draw each registered item at 0% before work starts.
            for bar in bars {
                bar.force_draw();
            }
            // Reduce flicker by moving the cursor instead of clearing lines once the item set is stable.
            self.multi.set_move_cursor(true);
        }

        fn finish(&self) {
            let bars = self.lock_bars().clone();

            for bar in bars {
                bar.finish_using_style();
            }
        }
    }

    fn bar_style() -> ProgressStyle {
        static STYLE: OnceLock<ProgressStyle> = OnceLock::new();
        STYLE
            .get_or_init(|| {
                let template = format!(
                    "{{msg:!{LABEL_WIDTH}}} [{{bar:{BAR_WIDTH}.cyan/blue}}] ({{percent:>3}}%)"
                );
                ProgressStyle::default_bar()
                    .template(&template)
                    .expect("progress bar template is valid")
                    .progress_chars(PROGRESS_CHARS)
            })
            .clone()
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
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("lineitem", 60);
            progress.increment(1);
            let progress = t.clone().register("orders", 15);
            progress.increment(5);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 1);
            assert_eq!(bars[1].position(), 5);
        }

        #[test]
        fn zero_total_items_start_at_zero() {
            let t = Arc::new(IndicatifProgress::hidden());
            t.clone().register("store_returns", 0);

            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 0);
            assert_eq!(bars[0].length(), Some(1));
            assert!(!bars[0].is_finished());
        }

        #[test]
        fn reaches_total() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("orders", 5);
            for _ in 0..5 {
                progress.increment(1);
            }
            let bars = t.bars.lock().unwrap();
            assert_eq!(bars[0].position(), 5);
            assert!(!bars[0].is_finished());
        }

        #[test]
        fn finish_marks_registered_items_finished() {
            let t = Arc::new(IndicatifProgress::hidden());
            let progress = t.clone().register("orders", 2);
            progress.increment(2);
            t.finish();

            assert!(t.bars.lock().unwrap()[0].is_finished());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    };

    /// Mock implementation that records every event. Demonstrates that
    /// the trait is dyn-compatible and usable from external code without
    /// pulling in `indicatif`.
    #[derive(Debug, Default)]
    struct MockTracker {
        registered: Mutex<Vec<(String, u64)>>,
        increments: Mutex<Vec<(String, u64)>>,
        finished: AtomicU64,
    }

    impl ProgressTracker for MockTracker {
        fn register(self: Arc<Self>, item: &str, total_units: u64) -> ProgressHandle {
            let item = item.to_owned();
            self.registered
                .lock()
                .unwrap()
                .push((item.clone(), total_units));
            ProgressHandle::new(move |units| {
                self.increments.lock().unwrap().push((item.clone(), units));
            })
        }
        fn finish(&self) {
            self.finished.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn mock_tracker_works_through_progress_handles() {
        let mock = Arc::new(MockTracker::default());
        let dynamic: Arc<dyn ProgressTracker> = mock.clone();
        let progress = dynamic.clone().register("store_sales", 10);
        progress.increment(3);
        let progress = dynamic.clone().register("catalog_returns", 4);
        progress.increment(1);
        dynamic.finish();

        assert_eq!(
            *mock.registered.lock().unwrap(),
            vec![
                ("store_sales".to_owned(), 10),
                ("catalog_returns".to_owned(), 4)
            ]
        );
        assert_eq!(
            *mock.increments.lock().unwrap(),
            vec![
                ("store_sales".to_owned(), 3),
                ("catalog_returns".to_owned(), 1)
            ]
        );
        assert_eq!(mock.finished.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn default_start_and_finish_are_noops() {
        #[derive(Debug)]
        struct Minimal(AtomicU64);
        impl ProgressTracker for Minimal {
            fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
                ProgressHandle::new(move |units| {
                    self.0.fetch_add(units, Ordering::Relaxed);
                })
            }
        }
        let m = Arc::new(Minimal(AtomicU64::new(0)));
        let progress = m.clone().register("region", 99);
        m.start(); // no-op default
        progress.increment(7);
        m.finish(); // no-op default
        assert_eq!(m.0.load(Ordering::Relaxed), 7);
    }
}
