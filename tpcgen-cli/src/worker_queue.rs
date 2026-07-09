//! [`WorkerQueue`]: run generation tasks in parallel within a thread budget.

use log::debug;
use std::future::Future;
use std::io;
use tokio::task::{JoinError, JoinSet};

/// Manages worker tasks, limiting the number of total outstanding threads
/// to some fixed number
///
/// Each task is run with a number of threads equal to the number of chunks
/// (e.g. parts or row groups) it will produce, but no more than the total
/// number of threads specified when creating the queue. If a task does not
/// need all the threads, the remaining threads are used to run other tasks.
///
/// This is important to keep all cores busy for smaller tables that may not
/// have sufficient chunks to keep all threads busy, but not schedule more
/// tasks than we have threads for.
///
/// Scheduling too many tasks requires more memory and leads to context
/// switching overhead, which can slow down the generation process.
pub(crate) struct WorkerQueue {
    join_set: JoinSet<io::Result<usize>>,
    /// Current number of threads available to commit
    available_threads: usize,
}

impl WorkerQueue {
    pub(crate) fn new(max_threads: usize) -> Self {
        assert!(max_threads > 0);
        Self {
            join_set: JoinSet::new(),
            available_threads: max_threads,
        }
    }

    /// Spawns a task with as many threads as possible without exceeding
    /// the maximum number of threads. The task is created by calling
    /// `task` with the number of threads allocated to it, and must return
    /// that number back when it completes so the threads can be reused.
    ///
    /// If there are no threads available, waits for a running task to
    /// finish before spawning the new one.
    ///
    /// Note this algorithm does not guarantee that all threads are always busy,
    /// but it should be good enough for most cases. For best thread utilization
    /// spawn the largest tasks first.
    pub(crate) async fn schedule<F, Fut>(&mut self, chunk_count: usize, task: F) -> io::Result<()>
    where
        F: FnOnce(usize) -> Fut,
        Fut: Future<Output = io::Result<usize>> + Send + 'static,
    {
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

            // figure out how many threads to allocate to this task. Each task
            // can use up to `chunk_count` threads.
            let num_task_threads = self.available_threads.min(chunk_count);

            debug!("Spawning task with {num_task_threads} threads");
            self.join_set.spawn(task(num_task_threads));
            self.available_threads -= num_task_threads;
            return Ok(());
        }
    }

    /// Wait for all tasks to finish
    pub(crate) async fn join_all(mut self) -> io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn schedules_more_tasks_than_threads() {
        let completed = Arc::new(AtomicUsize::new(0));
        let mut queue = WorkerQueue::new(2);

        for chunk_count in [3, 1, 1, 2, 1] {
            let completed = Arc::clone(&completed);
            queue
                .schedule(chunk_count, move |num_threads| async move {
                    // each task gets at least one thread and no more than
                    // its chunk count or the maximum
                    assert!(num_threads >= 1);
                    assert!(num_threads <= chunk_count.min(2));
                    completed.fetch_add(1, Ordering::Relaxed);
                    Ok(num_threads)
                })
                .await
                .unwrap();
        }
        queue.join_all().await.unwrap();

        assert_eq!(completed.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn task_error_is_propagated() {
        let mut queue = WorkerQueue::new(1);
        queue
            .schedule(1, |num_threads| async move {
                let _ = num_threads;
                Err(io::Error::other("boom"))
            })
            .await
            .unwrap();

        let err = queue.join_all().await.unwrap_err();
        assert!(err.to_string().contains("boom"));
    }
}
