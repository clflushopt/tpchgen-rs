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

pub mod call_center;
pub mod catalog_page;
pub mod catalog_returns;
pub mod catalog_sales;
pub mod conversions;
pub mod customer;
pub mod customer_address;
pub mod customer_demographics;
pub mod date_dim;
pub mod dbgen_version;
pub mod household_demographics;
pub mod income_band;
pub mod inventory;
pub mod item;
pub mod promotion;
pub mod reason;
pub mod ship_mode;
pub mod store;
pub mod store_returns;
pub mod store_sales;
pub mod time_dim;
pub mod warehouse;
pub mod web_page;
pub mod web_returns;
pub mod web_sales;
pub mod web_site;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;

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
