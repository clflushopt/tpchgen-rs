//! Progress tracking for table generation

use crate::Table;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Tracks progress for all tables being generated
#[derive(Clone, Debug)]
pub struct ProgressTracker {
    inner: Arc<ProgressTrackerInner>,
}

#[derive(Debug)]
struct ProgressTrackerInner {
    tables: Mutex<HashMap<Table, ProgressBar>>,
    // MultiProgress must be kept alive to manage the registered progress bars
    _multi_progress: MultiProgress,
}

impl ProgressTracker {
    /// Create a new progress tracker for the given tables
    pub fn new(tables: Vec<(Table, u64)>) -> Self {
        let multi_progress = MultiProgress::new();
        let mut table_map = HashMap::new();

        for (table, total_rows) in tables {
            let pb = multi_progress.add(ProgressBar::new(total_rows));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:10} [{bar:28}]   Progress: {percent:>3}%")
                    .unwrap()
                    .progress_chars("█▓░"),
            );
            pb.set_message(format!("{}", table));
            let pb = pb.with_finish(ProgressFinish::AndLeave);
            table_map.insert(table, pb);
        }

        Self {
            inner: Arc::new(ProgressTrackerInner {
                tables: Mutex::new(table_map),
                _multi_progress: multi_progress,
            }),
        }
    }

    pub fn increment(&self, table: Table, rows: u64) {
        let tables = self.inner.tables.lock().unwrap();
        if let Some(pb) = tables.get(&table) {
            pb.inc(rows);
        }
    }

    pub fn finish(&self, table: Table) {
        let tables = self.inner.tables.lock().unwrap();
        if let Some(pb) = tables.get(&table) {
            pb.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(vec![
            (Table::Lineitem, 60_000_000),
            (Table::Orders, 15_000_000),
        ]);
        tracker.increment(Table::Lineitem, 1_000_000);
    }

    #[test]
    fn test_progress_tracker_increment() {
        let tracker = ProgressTracker::new(vec![(Table::Customer, 150_000)]);
        for _ in 0..10 {
            tracker.increment(Table::Customer, 15_000);
        }
    }
}
