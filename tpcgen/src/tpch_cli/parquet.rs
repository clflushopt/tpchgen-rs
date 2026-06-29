//! Parquet output format

use crate::tpch_cli::progress::TableProgress;
use crate::tpch_cli::statistics::WriteStatistics;
use arrow::datatypes::SchemaRef;
use futures::StreamExt;
use log::debug;
use parquet::arrow::arrow_writer::{compute_leaves, ArrowColumnChunk};
use parquet::arrow::ArrowSchemaConverter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::types::SchemaDescPtr;
use std::io;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tpchgen_arrow::RecordBatchIterator;

pub trait IntoSize {
    /// Convert the object into a size
    fn into_size(self) -> Result<usize, io::Error>;
}

/// Converts a set of RecordBatchIterators into a Parquet file
///
/// Uses num_threads to generate the data in parallel
///
/// Note the input is an iterator of [`RecordBatchIterator`]; The batches
/// produced by each iterator is encoded as its own row group.
pub async fn generate_parquet<W: Write + Send + IntoSize + 'static, I>(
    writer: W,
    iter_iter: I,
    num_threads: usize,
    parquet_compression: Compression,
) -> Result<(), io::Error>
where
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    let progress = Default::default();
    generate_parquet_impl(
        writer,
        iter_iter,
        num_threads,
        parquet_compression,
        progress,
    )
    .await
}

pub(crate) async fn generate_parquet_with_progress<W: Write + Send + IntoSize + 'static, I>(
    writer: W,
    iter_iter: I,
    num_threads: usize,
    parquet_compression: Compression,
    progress: TableProgress,
) -> Result<(), io::Error>
where
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    generate_parquet_impl(
        writer,
        iter_iter,
        num_threads,
        parquet_compression,
        progress,
    )
    .await
}

async fn generate_parquet_impl<W: Write + Send + IntoSize + 'static, I>(
    writer: W,
    iter_iter: I,
    num_threads: usize,
    parquet_compression: Compression,
    progress: TableProgress,
) -> Result<(), io::Error>
where
    I: Iterator<Item: RecordBatchIterator> + 'static,
{
    debug!(
        "Generating Parquet with {num_threads} threads, using {parquet_compression} compression"
    );
    // Based on example in https://docs.rs/parquet/latest/parquet/arrow/arrow_writer/struct.ArrowColumnWriter.html
    let mut iter_iter = iter_iter.peekable();

    // get schema from the first iterator
    let Some(first_iter) = iter_iter.peek() else {
        return Ok(()); // no data shrug
    };
    let schema = Arc::clone(first_iter.schema());

    // Compute the parquet schema
    let writer_properties = WriterProperties::builder()
        .set_compression(parquet_compression)
        .build();
    let writer_properties = Arc::new(writer_properties);
    let parquet_schema = Arc::new(
        ArrowSchemaConverter::new()
            .with_coerce_types(writer_properties.coerce_types())
            .convert(&schema)
            .unwrap(),
    );

    // create a stream that computes the data for each row group
    let mut row_group_stream = futures::stream::iter(iter_iter)
        .map(async |iter| {
            let parquet_schema = Arc::clone(&parquet_schema);
            let writer_properties = Arc::clone(&writer_properties);
            let schema = Arc::clone(&schema);
            // run on a separate thread
            tokio::task::spawn(async move {
                encode_row_group(parquet_schema, writer_properties, schema, iter)
            })
            .await
            .expect("Inner task panicked")
        })
        .buffered(num_threads); // generate row groups in parallel

    let mut statistics = WriteStatistics::new("row groups");

    // A blocking task that writes the row groups to the file
    // done in a blocking task to avoid having a thread waiting on IO
    // Now, read each completed row group and write it to the file
    let root_schema = parquet_schema.root_schema_ptr();
    let writer_properties_captured = Arc::clone(&writer_properties);
    let (tx, mut rx): (
        Sender<Vec<ArrowColumnChunk>>,
        Receiver<Vec<ArrowColumnChunk>>,
    ) = tokio::sync::mpsc::channel(num_threads);
    let captured_progress = progress.clone();
    let writer_task = tokio::task::spawn_blocking(move || {
        // Create parquet writer
        let mut writer =
            SerializedFileWriter::new(writer, root_schema, writer_properties_captured).unwrap();

        while let Some(column_chunks) = rx.blocking_recv() {
            // Start row group
            let mut row_group_writer = writer.next_row_group().unwrap();

            // Slap the chunks into the row group
            for column_chunk in column_chunks {
                column_chunk
                    .append_to_row_group(&mut row_group_writer)
                    .unwrap();
            }
            row_group_writer.close().unwrap();
            statistics.increment_chunks(1);
            captured_progress.increment_output_unit();
        }
        let size = writer.into_inner()?.into_size()?;
        statistics.increment_bytes(size);
        Ok(()) as Result<(), io::Error>
    });

    // now, drive the input stream and send results to the writer task
    while let Some(column_chunks) = row_group_stream.next().await {
        // send the chunks to the writer task
        if let Err(e) = tx.send(column_chunks).await {
            debug!("Error sending row group to writer: {e}");
            break; // stop early
        }
    }
    // signal the writer task that we are done
    drop(tx);

    // Wait for the writer task to finish
    writer_task.await??;

    Ok(())
}

/// Creates the data for a particular row group
///
/// Note at the moment it does not use multiple tasks/threads but it could
/// potentially encode multiple columns with different threads .
///
/// Returns an array of [`ArrowColumnChunk`]
fn encode_row_group<I>(
    parquet_schema: SchemaDescPtr,
    writer_properties: Arc<WriterProperties>,
    schema: SchemaRef,
    iter: I,
) -> Vec<ArrowColumnChunk>
where
    I: RecordBatchIterator,
{
    // Create writers for each of the leaf columns
    #[allow(deprecated)]
    let mut col_writers = parquet::arrow::arrow_writer::get_column_writers(
        &parquet_schema,
        &writer_properties,
        &schema,
    )
    .unwrap();

    // generate the data and send it to the tasks (via the sender channels)
    for batch in iter {
        let columns = batch.columns().iter();
        let col_writers = col_writers.iter_mut();
        let fields = schema.fields().iter();

        for ((col_writer, field), arr) in col_writers.zip(fields).zip(columns) {
            for leaves in compute_leaves(field.as_ref(), arr).unwrap() {
                col_writer.write(&leaves).unwrap();
            }
        }
    }
    // finish the writers and create the column chunks
    col_writers
        .into_iter()
        .map(|col_writer| col_writer.close().unwrap())
        .collect()
}

#[cfg(all(test, feature = "progress"))]
mod tests {
    use super::*;
    use crate::tpch_cli::progress::{ProgressTracker, RunProgress};
    use crate::tpch_cli::Table;
    use std::fs::File;
    use std::io::BufWriter;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tpchgen::generators::RegionGenerator;
    use tpchgen_arrow::RegionArrow;

    #[derive(Debug)]
    struct CountingProgress {
        increments: AtomicU64,
    }

    impl ProgressTracker for CountingProgress {
        fn increment(&self, _table: Table, row_groups: u64) {
            self.increments.fetch_add(row_groups, Ordering::Relaxed);
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

        let tracker = Arc::new(CountingProgress {
            increments: AtomicU64::new(0),
        });
        let progress: Arc<dyn ProgressTracker> = tracker.clone();
        let progress = RunProgress::with_tracker(progress).for_table(Table::Region);

        generate_parquet_with_progress(
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
