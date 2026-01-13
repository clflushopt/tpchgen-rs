//! Parallel data generation: [`Source`] and [`Sink`] and [`generate_in_chunks`]
//!
//! These traits and function are used to generate data in parallel and write it to a sink
//! in streaming fashion (chunks). This is useful for generating large datasets that don't fit in memory.

use crate::progress::{IncrementType, ProgressTracker};
use crate::Table;
use futures::StreamExt;
use log::debug;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;

/// Something that knows how to generate data into a buffer
///
/// For example, this is implemented for the different generators in the tpchgen
/// crate
pub trait Source: Send {
    /// generates the data for this generator into the buffer, returning the buffer.
    fn create(self, buffer: Vec<u8>) -> Vec<u8>;

    /// Create the first line for the output, into the buffer
    ///
    /// This will be called before the first call to [`Self::create`] and
    /// exactly once across all [`Source`]es
    fn header(&self, buffer: Vec<u8>) -> Vec<u8>;
}

/// Something that can write the contents of a buffer somewhere
///
/// For example, this is implemented for a file writer.
pub trait Sink: Send {
    /// Write all data from the buffer to the sink
    fn sink(&mut self, buffer: &[u8]) -> Result<(), io::Error>;

    /// Complete and flush any remaining data from the sink
    fn flush(self) -> Result<(), io::Error>;
}

/// Generates data in parallel from a series of [`Source`] and writes to a [`Sink`]
///
/// Each [`Source`] is a data generator that generates data directly into an in
/// memory buffer.
///
/// This function will run the [`Source`]es in parallel up to num_threads.
/// Data is written to the [`Sink`] in the order of the [`Source`]es in
/// the input iterator.
///
/// G: Generator
/// I: Iterator<Item = G>
/// S: Sink that writes buffers somewhere
pub async fn generate_in_chunks<G, I, S>(
    mut sink: S,
    sources: I,
    num_threads: usize,
    progress_tracker: Option<ProgressTracker>,
    table: Table,
) -> Result<(), io::Error>
where
    G: Source + 'static,
    I: Iterator<Item = G>,
    S: Sink + 'static,
{
    let recycler = BufferRecycler::new();
    let mut sources = sources.peekable();

    // use all cores to make data
    debug!("Using {num_threads} threads");

    // create a channel to communicate between the generator tasks and the writer task
    let (tx, mut rx) = tokio::sync::mpsc::channel(num_threads);

    // write the header
    let Some(first) = sources.peek() else {
        return Ok(()); // no sources
    };
    let header = first.header(Vec::new());
    tx.send(header)
        .await
        .expect("tx just created, it should not be closed");

    let sources_and_recyclers = sources.map(|generator| (generator, recycler.clone()));

    // convert to an async stream to run on tokio
    let mut stream = futures::stream::iter(sources_and_recyclers)
        // each generator writes to a buffer
        .map(|(source, recycler)| async move {
            let buffer = recycler.new_buffer(1024 * 1024 * 8);
            // do the work in a task (on a different thread)
            let mut join_set = JoinSet::new();
            join_set.spawn(async move { source.create(buffer) });
            // wait for the task to be done and return the result
            join_set
                .join_next()
                .await
                .expect("had one item")
                .expect("join_next join is infallible unless task panics")
        })
        // run in parallel
        .buffered(num_threads)
        .map(async |buffer| {
            // send the buffer to the writer task, in order.

            // Note we ignore errors writing because if the write errors it
            // means the channel is closed / the program is exiting so there
            // is nothing listening to send errors
            if let Err(e) = tx.send(buffer).await {
                debug!("Error sending buffer to writer: {e}");
            }
        });

    // The writer task runs in a blocking thread to avoid blocking the async
    // runtime. It reads from the channel and writes to the sink (doing File IO)
    let captured_recycler = recycler.clone();
    let captured_progress_for_buffers = progress_tracker.clone();
    let writer_task = tokio::task::spawn_blocking(move || {
        while let Some(buffer) = rx.blocking_recv() {
            sink.sink(&buffer)?;
            captured_recycler.return_buffer(buffer);
            // Increment buffer count after each buffer/chunk is written
            if let Some(ref tracker) = captured_progress_for_buffers {
                tracker.increment(table, IncrementType::Buffer);
            }
        }
        // No more input, flush the sink and return
        sink.flush()
    });

    // drive the stream to completion
    while let Some(write_task) = stream.next().await {
        // break early if the writer stream is done (errored)
        if writer_task.is_finished() {
            debug!("writer task is done early, stopping writer");
            break;
        }
        write_task.await; // sends the buffer to the writer task
    }
    drop(stream); // drop any stream references
    drop(tx); // drop last tx reference to tell the writer it is done.

    // wait for writer to finish
    debug!("waiting for writer task to complete");
    writer_task.await.expect("writer task panicked")
}

/// A simple buffer recycler to avoid allocating new buffers for each part
///
/// Clones share the same underlying recycler, so it is not thread safe
#[derive(Debug, Clone)]
struct BufferRecycler {
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl BufferRecycler {
    fn new() -> Self {
        Self {
            buffers: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    /// return a new empty buffer, with size bytes capacity
    fn new_buffer(&self, size: usize) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        if let Some(mut buffer) = buffers.pop_front() {
            buffer.clear();
            if size > buffer.capacity() {
                buffer.reserve(size - buffer.capacity());
            }
            buffer
        } else {
            Vec::with_capacity(size)
        }
    }

    fn return_buffer(&self, buffer: Vec<u8>) {
        let mut buffers = self.buffers.lock().unwrap();
        buffers.push_back(buffer);
    }
}
