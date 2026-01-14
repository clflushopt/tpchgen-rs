//! Progress tracking for table generation

use crate::Table;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Type of progress increment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementType {
    /// Increment the number of parts/files completed
    Part,
    /// Increment the number of chunks/buffers written
    Buffer,
}

/// Tracks progress for all tables being generated
#[derive(Clone, Debug)]
pub struct ProgressTracker {
    inner: Arc<ProgressTrackerInner>,
}

#[derive(Debug)]
struct ProgressTrackerInner {
    tables: Mutex<HashMap<Table, TableProgress>>,
    // MultiProgress is unused but we need to it alive to
    // manage the registered progress bars.
    #[allow(dead_code)]
    multi_progress: MultiProgress,
}

#[derive(Debug)]
struct TableProgress {
    parts_completed: AtomicUsize,
    buffers_written: AtomicUsize,
    progress_bar: ProgressBar,
}

impl ProgressTracker {
    /// Create a new progress tracker for the given tables
    pub fn new(tables: Vec<(Table, usize)>) -> Self {
        let multi_progress = MultiProgress::new();
        let mut table_map = HashMap::new();

        for (table, total_parts) in tables {
            let mut pb = multi_progress.add(ProgressBar::new(total_parts as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:10} [{bar:28}] Parts:{pos:>4}/{len:<4} Buffers:{prefix:>6} {percent:>3}%")
                    .unwrap()
                    .progress_chars("█▓░")
            );
            pb.set_message(format!("{}", table));
            pb.set_prefix("0");
            // Configure to leave the progress bar visible after finishing
            pb = pb.with_finish(ProgressFinish::AndLeave);

            table_map.insert(
                table,
                TableProgress {
                    parts_completed: AtomicUsize::new(0),
                    buffers_written: AtomicUsize::new(0),
                    progress_bar: pb,
                },
            );
        }

        Self {
            inner: Arc::new(ProgressTrackerInner {
                tables: Mutex::new(table_map),
                multi_progress,
            }),
        }
    }

    /// Increment progress counter for a table
    pub fn increment(&self, table: Table, increment_type: IncrementType) {
        let tables = self.inner.tables.lock().unwrap();
        if let Some(progress) = tables.get(&table) {
            match increment_type {
                IncrementType::Part => {
                    let new_val = progress.parts_completed.fetch_add(1, Ordering::SeqCst) + 1;
                    progress.progress_bar.set_position(new_val as u64);
                }
                IncrementType::Buffer => {
                    let new_val = progress.buffers_written.fetch_add(1, Ordering::SeqCst) + 1;
                    progress.progress_bar.set_prefix(format!("{}", new_val));
                }
            }
        }
    }

    /// Mark a table as complete (progress bar will stay visible due to ProgressFinish::AndLeave)
    pub fn finish(&self, table: Table) {
        let tables = self.inner.tables.lock().unwrap();
        if let Some(progress) = tables.get(&table) {
            progress.progress_bar.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(vec![(Table::Lineitem, 10), (Table::Orders, 5)]);

        // Test that we can increment without panicking
        tracker.increment(Table::Lineitem, IncrementType::Part);
        tracker.increment(Table::Orders, IncrementType::Buffer);
    }

    #[test]
    fn test_progress_tracker_increment() {
        let tracker = ProgressTracker::new(vec![(Table::Customer, 4)]);

        // Increment parts and buffers
        for _ in 0..3 {
            tracker.increment(Table::Customer, IncrementType::Part);
        }
        for _ in 0..10 {
            tracker.increment(Table::Customer, IncrementType::Buffer);
        }

        // Verify the counts
        let tables = tracker.inner.tables.lock().unwrap();
        let progress = tables.get(&Table::Customer).unwrap();
        assert_eq!(progress.parts_completed.load(Ordering::SeqCst), 3);
        assert_eq!(progress.buffers_written.load(Ordering::SeqCst), 10);
    }
}
