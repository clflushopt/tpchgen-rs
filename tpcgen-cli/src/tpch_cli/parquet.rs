//! TPCH Parquet output format.

use crate::tpch_cli::progress::{no_op_progress_tracker, ProgressTracker};
use parquet::basic::Compression;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tpchgen_arrow::RecordBatchIterator;

pub use crate::parquet::IntoSize;

/// Converts a set of RecordBatchIterators into a Parquet file.
///
/// Uses num_threads to generate the data in parallel.
///
/// Note the input is an iterator of [`RecordBatchIterator`]; the batches
/// produced by each iterator are encoded as their own row group.
pub async fn generate_parquet<W, I>(
    writer: W,
    iter_iter: I,
    num_threads: usize,
    parquet_compression: Compression,
    progress: Arc<dyn ProgressTracker>,
    table_name: &'static str,
) -> Result<(), io::Error>
where
    W: Write + Send + IntoSize + 'static,
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    let mut iter_iter = iter_iter.peekable();
    let Some(first_iter) = iter_iter.peek() else {
        return Ok(());
    };
    let schema = Arc::clone(first_iter.schema());

    crate::parquet::generate_parquet(
        writer,
        schema,
        iter_iter,
        num_threads,
        parquet_compression,
        progress,
        table_name,
    )
    .await
}
