//! Parallel data generation: [`Source`] and [`Sink`] and [`generate_in_chunks`]
//!
//! These traits and function are used to generate data in parallel and write it to a sink
//! in streaming fashion (chunks). This is useful for generating large datasets that don't fit in memory.

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
}

/// Something that can write the contents of a buffer somewhere
///
/// For example, this is implemented for a file writer
pub trait Sink: Send {
    /// Write all data from the buffer to the sink
    fn sink(&mut self, buffer: &[u8]) -> Result<(), io::Error>;

    fn finish(self) -> Result<(), io::Error>;
}

/// Creates data from a set of [`Source`] in parallel sending data in order to the Sink.
///
///
/// G: Generator
/// I: Iterator<Item = G>
/// S: Sink that writes buffers somewhere
pub async fn generate_in_chunks<G, I, S>(mut sink: S, generators: I) -> Result<(), io::Error>
where
    G: Source + 'static,
    I: Iterator<Item = G>,
    S: Sink + 'static,
{
    let recycler = BufferRecycler::new();

    // use all cores to make data
    let num_tasks = num_cpus::get();
    debug!("Generating with {num_tasks} parallel tasks");

    // create a channel to communicate between the generator tasks and the writer task
    let (tx, mut rx) = tokio::sync::mpsc::channel(num_tasks);

    let generators_and_recyclers = generators.map(|generator| (generator, recycler.clone()));

    // convert to an async stream to run on tokio
    let mut stream = futures::stream::iter(generators_and_recyclers)
        // each generator writes to a buffer
        .map(async |(generator, recycler)| {
            let buffer = recycler.new_buffer(1024 * 1024 * 8);
            // do the work in a task (on a different thread)
            let mut join_set = JoinSet::new();
            join_set.spawn(async move { generator.create(buffer) });
            // wait for the task to be done and return the result
            join_set
                .join_next()
                .await
                .expect("had one item")
                .expect("join_next join is infallible unless task panics")
        })
        // run in parallel
        .buffered(num_tasks)
        .map(async |buffer| {
            // send the buffer to the writer task, in order.

            // ignoring error (if the writer errored it means the channel is closed / the program is exiting)
            if let Err(e) = tx.send(buffer).await {
                debug!("Error sending buffer to writer: {e}");
            }
        });

    let captured_recycler = recycler.clone();
    let writer_task = tokio::task::spawn_blocking(move || {
        while let Some(buffer) = rx.blocking_recv() {
            sink.sink(&buffer)?;
            captured_recycler.return_buffer(buffer);
        }
        sink.finish()
    });

    // drive the stream to completion
    while let Some(write_task) = stream.next().await {
        write_task.await; // sends the buffer to the writer task
    }
    drop(stream); // drop any stream references
    drop(tx); // drop last tx reference to stop the writer

    // wait for writer to finish
    debug!("waiting for writer task to complete");
    writer_task.await.expect("writer task panicked")
}

/// A simple buffer recycler to avoid allocating new buffers for each part
///
/// Clones share the same undrlying recycler, so it is not thread safe
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
