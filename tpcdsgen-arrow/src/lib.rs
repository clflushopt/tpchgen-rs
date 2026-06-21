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

mod call_center;
mod catalog_page;
mod catalog_returns;
mod catalog_sales;
pub mod conversions;
mod customer;
mod customer_address;
mod customer_demographics;
mod date_dim;
mod dbgen_version;
mod household_demographics;
mod income_band;
mod inventory;
mod item;
mod promotion;
mod reason;
mod ship_mode;
mod store;
mod store_returns;
mod store_sales;
mod time_dim;
mod warehouse;
mod web_page;
mod web_returns;
mod web_sales;
mod web_site;

use std::collections::VecDeque;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use tpcdsgen::config::Session;
use tpcdsgen::row::{GeneratedRow, RowGenerator};

pub use call_center::CallCenterArrow;
pub use catalog_page::CatalogPageArrow;
pub use catalog_returns::CatalogReturnsArrow;
pub use catalog_sales::CatalogSalesArrow;
pub use customer::CustomerArrow;
pub use customer_address::CustomerAddressArrow;
pub use customer_demographics::CustomerDemographicsArrow;
pub use date_dim::DateDimArrow;
pub use dbgen_version::DbgenVersionArrow;
pub use household_demographics::HouseholdDemographicsArrow;
pub use income_band::IncomeBandArrow;
pub use inventory::InventoryArrow;
pub use item::ItemArrow;
pub use promotion::PromotionArrow;
pub use reason::ReasonArrow;
pub use ship_mode::ShipModeArrow;
pub use store::StoreArrow;
pub use store_returns::StoreReturnsArrow;
pub use store_sales::StoreSalesArrow;
pub use time_dim::TimeDimArrow;
pub use warehouse::WarehouseArrow;
pub use web_page::WebPageArrow;
pub use web_returns::WebReturnsArrow;
pub use web_sales::WebSalesArrow;
pub use web_site::WebSiteArrow;

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
        Self { generator, session, current_row: 1, row_count, pending: VecDeque::new() }
    }
}

impl<G: RowGenerator> Iterator for RowIter<G> {
    type Item = GeneratedRow;

    fn next(&mut self) -> Option<GeneratedRow> {
        while self.pending.is_empty() {
            if self.current_row > self.row_count {
                return None;
            }
            let result = self.generator
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
