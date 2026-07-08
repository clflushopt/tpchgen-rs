//! TPCH Parquet output format.

use crate::progress::ProgressTracker;
use arrow::record_batch::RecordBatchReader;
use parquet::basic::Compression;
use std::io;
use std::io::Write;
use std::sync::Arc;

pub use crate::parquet::IntoSize;

/// Converts a set of RecordBatchReaders into a Parquet file.
///
/// Uses num_threads to generate the data in parallel.
///
/// Note the input is an iterator of [`RecordBatchReader`]; the batches
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
    I: Iterator + 'static,
    I::Item: RecordBatchReader + Send,
{
    let mut iter_iter = iter_iter.peekable();
    let Some(first_iter) = iter_iter.peek() else {
        return Ok(());
    };
    let schema = first_iter.schema();

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
