//! Generate TPC-DS data as Apache Arrow [`RecordBatch`]es.
//!
//! This crate wraps the [`tpcdsgen`] row generators and produces typed Arrow
//! arrays directly — bypassing the intermediate string formatting step —
//! for significantly faster ingestion into Arrow-based engines.
//!
//! # Example
//! ```
//! use tpcdsgen::config::Options;
//! use tpcdsgen_arrow::{ReasonArrow, RecordBatchIterator};
//!
//! let session = Options::default().to_session().unwrap();
//! let mut gen = ReasonArrow::new(session).with_batch_size(100);
//! let batch = gen.next().unwrap();
//! assert_eq!(batch.num_columns(), 3);
//! ```

pub mod conversions;
mod tables;

use std::collections::VecDeque;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use tpcdsgen::config::Session;
use tpcdsgen::row::{GeneratedRow, RowGenerator};

pub use tables::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

/// An iterator of Arrow [`RecordBatch`]es that also exposes its schema.
pub trait RecordBatchIterator: Iterator<Item = RecordBatch> + Send {
    fn schema(&self) -> &SchemaRef;
}

/// Default number of rows per [`RecordBatch`].
pub const DEFAULT_BATCH_SIZE: usize = 8_000;

/// Adapts a [`RowGenerator`] into a streaming [`Iterator`] of [`GeneratedRow`]s.
///
/// Handles both simple generators (one row per call, `should_end_row` always
/// true) and paired fact-table generators (multiple calls per source row,
/// `should_end_row` signals when to advance the row counter).
pub(crate) struct RowIter<G: RowGenerator> {
    generator: G,
    session: Session,
    current_row: i64,
    row_count: i64,
    pending: VecDeque<GeneratedRow>,
}

impl<G: RowGenerator> RowIter<G> {
    pub(crate) fn new(generator: G, session: Session, row_count: i64) -> Self {
        Self {
            generator,
            session,
            current_row: 1,
            row_count,
            pending: VecDeque::new(),
        }
    }
}

impl<G: RowGenerator> Iterator for RowIter<G> {
    type Item = GeneratedRow;

    fn next(&mut self) -> Option<GeneratedRow> {
        while self.pending.is_empty() {
            if self.current_row > self.row_count {
                return None;
            }
            let result = self
                .generator
                .generate_row_and_child_rows(self.current_row, &self.session, None, None)
                .expect("row gen");
            for row in result.get_rows() {
                self.pending.push_back(row.clone());
            }
            if result.should_end_row() {
                self.generator.consume_remaining_seeds_for_row();
                self.current_row += 1;
            }
        }
        self.pending.pop_front()
    }
}
