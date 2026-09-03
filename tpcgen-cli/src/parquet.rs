//! Shared Parquet output helpers.

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatchReader;
use futures::StreamExt;
use log::debug;
use parquet::arrow::arrow_writer::{compute_leaves, ArrowColumnChunk, ArrowRowGroupWriterFactory};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::progress::ProgressHandle;
use crate::tpch_cli::statistics::WriteStatistics;

type EncodedRowGroup = Vec<ArrowColumnChunk>;

pub trait IntoSize {
    /// Convert the object into a size
    fn into_size(self) -> Result<usize, io::Error>;
}

/// Converts a set of RecordBatchReaders into a Parquet file.
///
/// Uses num_threads to generate the data in parallel.
///
/// Note the input is an iterator of [`RecordBatchReader`]s; the batches
/// produced by each iterator are encoded as their own row group.
pub async fn generate_parquet<W, I>(
    writer: W,
    readers: I,
    num_threads: usize,
    parquet_compression: Compression,
    progress: ProgressHandle,
) -> Result<(), io::Error>
where
    W: Write + Send + IntoSize + 'static,
    I: Iterator + 'static,
    I::Item: RecordBatchReader + Send,
{
    debug!(
        "Generating Parquet with {num_threads} threads, using {parquet_compression} compression"
    );
    let mut readers = readers.peekable();
    let Some(first_reader) = readers.peek() else {
        return Ok(()); // no data
    };
    let schema = first_reader.schema();

    let writer_properties = WriterProperties::builder()
        .set_compression(parquet_compression)
        .build();
    // Start with ArrowWriter so schema conversion and Arrow metadata use the
    // standard path, then split it into the file writer and row-group factory.
    let writer = ArrowWriter::try_new(writer, Arc::clone(&schema), Some(writer_properties))
        .map_err(io::Error::other)?;
    let (writer, row_group_factory) = writer.into_serialized_writer().map_err(io::Error::other)?;
    let row_group_factory = Arc::new(row_group_factory);

    // Fan out: encode row groups concurrently while preserving input order.
    let mut encoded_row_groups = futures::stream::iter(readers.enumerate())
        .map(async |(row_group_index, reader)| {
            let row_group_factory = Arc::clone(&row_group_factory);
            let schema = Arc::clone(&schema);
            tokio::task::spawn_blocking(move || {
                encode_row_group(
                    row_group_factory.as_ref(),
                    row_group_index,
                    schema.as_ref(),
                    reader,
                )
            })
            .await
            .map_err(|e| io::Error::other(format!("Inner task panicked: {e}")))?
        })
        .buffered(num_threads);

    // Fan in: a single blocking task appends encoded row groups in order.
    let (tx, rx): (Sender<EncodedRowGroup>, Receiver<EncodedRowGroup>) =
        tokio::sync::mpsc::channel(num_threads);
    let writer_task = tokio::task::spawn_blocking(move || write_row_groups(writer, rx, progress));

    while let Some(row_group) = encoded_row_groups.next().await {
        let row_group = row_group?;
        if let Err(e) = tx.send(row_group).await {
            debug!("Error sending row group to writer: {e}");
            break;
        }
    }
    drop(tx);
    writer_task.await??;

    Ok(())
}

fn write_row_groups<W>(
    mut writer: SerializedFileWriter<W>,
    mut row_groups: Receiver<EncodedRowGroup>,
    progress: ProgressHandle,
) -> Result<(), io::Error>
where
    W: Write + Send + IntoSize,
{
    let mut statistics = WriteStatistics::new("row groups");

    while let Some(column_chunks) = row_groups.blocking_recv() {
        let mut row_group_writer = writer.next_row_group().map_err(io::Error::other)?;
        for column_chunk in column_chunks {
            column_chunk
                .append_to_row_group(&mut row_group_writer)
                .map_err(io::Error::other)?;
        }
        row_group_writer.close().map_err(io::Error::other)?;
        statistics.increment_chunks(1);
        progress.increment(1);
    }

    let size = writer.into_inner().map_err(io::Error::other)?.into_size()?;
    statistics.increment_bytes(size);
    Ok(())
}

/// Encodes all batches from one reader as a Parquet row group.
fn encode_row_group<I>(
    row_group_factory: &ArrowRowGroupWriterFactory,
    row_group_index: usize,
    schema: &Schema,
    reader: I,
) -> Result<EncodedRowGroup, io::Error>
where
    I: RecordBatchReader,
{
    let mut col_writers = row_group_factory
        .create_column_writers(row_group_index)
        .map_err(io::Error::other)?;

    for batch in reader {
        let batch = batch.map_err(io::Error::other)?;
        let columns = batch.columns().iter();
        let col_writers = col_writers.iter_mut();
        let fields = schema.fields().iter();

        for ((col_writer, field), arr) in col_writers.zip(fields).zip(columns) {
            for leaves in compute_leaves(field.as_ref(), arr).map_err(io::Error::other)? {
                col_writer.write(&leaves).map_err(io::Error::other)?;
            }
        }
    }
    col_writers
        .into_iter()
        .map(|col_writer| col_writer.close().map_err(io::Error::other))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::{ProgressHandle, ProgressTracker};
    use std::fs::File;
    use std::io::BufWriter;
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };
    use tpchgen::generators::RegionGenerator;
    use tpchgen_arrow::RegionArrow;

    #[derive(Debug, Default)]
    struct CountingProgress {
        increments: AtomicU64,
    }

    impl ProgressTracker for CountingProgress {
        fn register(self: Arc<Self>, _item: &str, _total_units: u64) -> ProgressHandle {
            ProgressHandle::new(move |row_groups| {
                self.increments.fetch_add(row_groups, Ordering::Relaxed);
            })
        }
    }

    fn region_source() -> RegionArrow {
        RegionArrow::new(RegionGenerator::default()).with_batch_size(5)
    }

    #[tokio::test]
    async fn progress_counts_written_row_groups() {
        let output_dir = tempfile::tempdir().unwrap();
        let output_path = output_dir.path().join("progress.parquet");
        let writer = BufWriter::new(File::create(&output_path).unwrap());

        let tracker = Arc::new(CountingProgress::default());
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let progress = progress.register("ignored", 2);

        generate_parquet(
            writer,
            vec![region_source(), region_source()].into_iter(),
            1,
            Compression::UNCOMPRESSED,
            progress,
        )
        .await
        .unwrap();

        assert_eq!(tracker.increments.load(Ordering::Relaxed), 2);
        assert!(std::fs::metadata(output_path).unwrap().len() > 0);
    }
}
