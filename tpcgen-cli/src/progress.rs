//! Progress tracking for data generation.
//!
//! # Overview
//!
//! [`ProgressTracker`] is a small, dyn-compatible trait that receives
//! generation events. Generation code emits these events as applicable:
//!
//! 1. [`ProgressTracker::register`] once per progress item, before work
//!    starts, with the total number of output units the item will produce
//!    (chunks for TBL/CSV, row groups for Parquet).
//! 2. [`ProgressTracker::start`] once after all known progress items have
//!    been registered and before work starts. This hook is optional for
//!    paths that register items lazily.
//! 3. [`ProgressTracker::increment`] after output units are written.
//!    Multiple generation tasks may call it concurrently, so impls
//!    must be `Send + Sync` and `increment` itself should be lightweight.
//! 4. [`ProgressTracker::finish`] once on the success path when the
//!    generation run exits. Implementations should use `finish` for normal
//!    success cleanup and `Drop` only as an error or panic fallback.
//!
//! When invoked, `register`, `start`, and `finish` are serial and may
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
//! use tpcgen_cli::progress::ProgressTracker;
//!
//! #[derive(Debug)]
//! struct LoggingTracker {
//!     written: AtomicU64,
//! }
//!
//! impl ProgressTracker for LoggingTracker {
//!     fn register(&self, item: &str, total: u64) {
//!         eprintln!("plan: {item} -> {total} output units");
//!     }
//!     fn increment(&self, _item: &str, units: u64) {
//!         self.written.fetch_add(units, Ordering::Relaxed);
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
/// concurrent generation tasks through [`ProgressHandle`]s, so they must be
/// `Send + Sync`.
/// They must also be `Debug` so containing types can derive `Debug`.
pub trait ProgressTracker: Send + Sync + fmt::Debug {
    /// Pre-register a progress item with its total expected output-unit count.
    ///
    /// Called once per item before work starts. The `item` is a stable
    /// identifier, usually a table name. Implementations
    /// that need to know totals up front (e.g. to render a progress bar
    /// or compute an ETA) should override this; the default does
    /// nothing.
    fn register(&self, _item: &str, _total_units: u64) {}

    /// Optional hook called after all known progress items have been
    /// registered and before the first [`Self::increment`].
    ///
    /// Implementations can use this to finalize setup that depends on the
    /// registered item set. The default does nothing.
    fn start(&self) {}

    /// Advance the counter for `item` by `units` output units.
    ///
    /// Called after generated output units are written. Multiple
    /// generation tasks may call this concurrently, so implementations
    /// should be lightweight and must never panic.
    fn increment(&self, item: &str, units: u64);

    /// Called once after the last [`Self::increment`] on the success
    /// path. Implementations should use this for normal success cleanup
    /// and `Drop` only as an error or panic fallback. The default does
    /// nothing.
    fn finish(&self) {}
}

/// A cloneable handle for reporting progress for one registered item.
///
/// This binds a [`ProgressTracker`] to a stable item identifier so generation
/// code only needs to carry one value and call [`Self::increment`]. Run-level
/// lifecycle operations such as registration, startup, and completion remain
/// the responsibility of the tracker owner.
#[derive(Clone, Debug)]
pub struct ProgressHandle {
    tracker: Arc<dyn ProgressTracker>,
    item: &'static str,
}

impl ProgressHandle {
    /// Create a handle for `item` backed by `tracker`.
    pub fn new(tracker: Arc<dyn ProgressTracker>, item: &'static str) -> Self {
        Self { tracker, item }
    }

    /// Advance this item's counter by `units` output units.
    pub fn increment(&self, units: u64) {
        self.tracker.increment(self.item, units);
    }
}

/// Default tracker used when no progress backend is installed.
///
/// This keeps generation on the same always-reporting path; the no-op
/// implementation simply ignores every event.
#[derive(Debug, Default)]
struct NoOpProgressTracker;

impl ProgressTracker for NoOpProgressTracker {
    fn increment(&self, _item: &str, _units: u64) {}
}

pub(crate) fn no_op_progress_tracker() -> Arc<dyn ProgressTracker> {
    Arc::new(NoOpProgressTracker)
}

#[cfg(feature = "indicatif-progress")]
pub use indicatif_impl::IndicatifProgress;

#[cfg(feature = "indicatif-progress")]
mod indicatif_impl {
    use super::ProgressTracker;
    use indicatif::ProgressDrawTarget;
    use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
    use std::collections::BTreeMap;
    use std::io::{self, Write};
    use std::sync::{OnceLock, RwLock};

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
    /// Items are added in [`ProgressTracker::register`] and looked up by item
    /// identifier on each [`ProgressTracker::increment`] call.
    #[derive(Debug)]
    pub struct IndicatifProgress {
        multi: MultiProgress,
        items: RwLock<BTreeMap<String, ProgressBar>>,
    }

    impl IndicatifProgress {
        /// Construct an empty tracker. Progress items are added via
        /// [`ProgressTracker::register`].
        pub fn new() -> Self {
            Self {
                multi: MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(
                    PROGRESS_REFRESH_HZ,
                )),
                items: RwLock::new(BTreeMap::new()),
            }
        }

        /// Return a writer that coordinates stderr log writes with progress
        /// bar redraws.
        pub fn log_writer(&self) -> Box<dyn io::Write + Send + 'static> {
            Box::new(IndicatifLogWriter {
                multi: self.multi.clone(),
            })
        }

        #[cfg(test)]
        fn hidden() -> Self {
            Self {
                multi: MultiProgress::with_draw_target(ProgressDrawTarget::hidden()),
                items: RwLock::new(BTreeMap::new()),
            }
        }
    }

    impl Default for IndicatifProgress {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProgressTracker for IndicatifProgress {
        fn register(&self, item: &str, total_units: u64) {
            let Ok(mut items) = self.items.write() else {
                return;
            };

            // Indicatif treats zero-length items as complete.
            let bar_len = total_units.max(1);
            let item_key = item.to_owned();
            let pb = self.multi.add(
                ProgressBar::new(bar_len)
                    .with_style(bar_style())
                    .with_message(item_key.clone())
                    .with_finish(ProgressFinish::AndLeave),
            );
            items.insert(item_key, pb);
        }

        fn start(&self) {
            let bars = {
                let Ok(items) = self.items.read() else {
                    return;
                };
                items.values().cloned().collect::<Vec<_>>()
            };

            // Draw each registered item at 0% before work starts.
            for bar in bars {
                bar.force_draw();
            }
            // Reduce flicker by moving the cursor instead of clearing lines once the item set is stable.
            self.multi.set_move_cursor(true);
        }

        fn increment(&self, item: &str, units: u64) {
            let bar = {
                let Ok(items) = self.items.read() else {
                    return;
                };
                items.get(item).cloned()
            };

            if let Some(bar) = bar {
                bar.inc(units);
            }
        }

        fn finish(&self) {
            let bars = {
                let Ok(items) = self.items.read() else {
                    return;
                };
                items.values().cloned().collect::<Vec<_>>()
            };

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
            let t = IndicatifProgress::hidden();
            t.register("lineitem", 60);
            t.register("orders", 15);
            t.increment("lineitem", 1);
            t.increment("orders", 5);

            let items = t.items.read().unwrap();
            assert_eq!(items["lineitem"].position(), 1);
            assert_eq!(items["orders"].position(), 5);
        }

        #[test]
        fn zero_total_items_start_at_zero() {
            let t = IndicatifProgress::hidden();
            t.register("store_returns", 0);

            let items = t.items.read().unwrap();
            assert_eq!(items["store_returns"].position(), 0);
            assert_eq!(items["store_returns"].length(), Some(1));
            assert!(!items["store_returns"].is_finished());
        }

        #[test]
        fn reaches_total() {
            let t = IndicatifProgress::hidden();
            t.register("orders", 5);
            for _ in 0..5 {
                t.increment("orders", 1);
            }
            let items = t.items.read().unwrap();
            assert_eq!(items["orders"].position(), 5);
            assert!(!items["orders"].is_finished());
        }

        #[test]
        fn unknown_item_is_no_op() {
            // Incrementing an item not registered must not panic.
            let t = IndicatifProgress::hidden();
            t.register("orders", 1);
            t.increment("lineitem", 1);
            assert_eq!(t.items.read().unwrap()["orders"].position(), 0);
        }

        #[test]
        fn finish_marks_registered_items_finished() {
            let t = IndicatifProgress::hidden();
            t.register("orders", 2);
            t.increment("orders", 2);
            t.finish();

            assert!(t.items.read().unwrap()["orders"].is_finished());
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
        fn register(&self, item: &str, total_units: u64) {
            self.registered
                .lock()
                .unwrap()
                .push((item.to_owned(), total_units));
        }
        fn increment(&self, item: &str, units: u64) {
            self.increments
                .lock()
                .unwrap()
                .push((item.to_owned(), units));
        }
        fn finish(&self) {
            self.finished.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn mock_tracker_works_through_progress_handles() {
        let mock = Arc::new(MockTracker::default());
        let dynamic: Arc<dyn ProgressTracker> = mock.clone();
        dynamic.register("store_sales", 10);
        dynamic.register("catalog_returns", 4);
        let store_sales = ProgressHandle::new(Arc::clone(&dynamic), "store_sales");
        let catalog_returns = ProgressHandle::new(Arc::clone(&dynamic), "catalog_returns");
        store_sales.increment(3);
        catalog_returns.increment(1);
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
    fn default_register_and_finish_are_noops() {
        // An impl that only overrides `increment` should compile and run.
        #[derive(Debug)]
        struct Minimal(AtomicU64);
        impl ProgressTracker for Minimal {
            fn increment(&self, _item: &str, units: u64) {
                self.0.fetch_add(units, Ordering::Relaxed);
            }
        }
        let m = Minimal(AtomicU64::new(0));
        m.register("region", 99); // no-op default
        m.increment("region", 7);
        m.finish(); // no-op default
        assert_eq!(m.0.load(Ordering::Relaxed), 7);
    }
}
