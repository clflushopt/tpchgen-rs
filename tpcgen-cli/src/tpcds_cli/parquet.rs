//! TPC-DS Parquet output.

use super::plan::TpcdsGenerationPlan;
use crate::parquet::generate_parquet;
use crate::progress::ProgressTracker;
use arrow::record_batch::RecordBatchReader;
use parquet::basic::Compression;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Parquet output generator.
#[derive(Debug, Clone)]
pub(super) struct Parquet {
    output_dir: PathBuf,
    compression: Compression,
    row_group_bytes: usize,
    num_threads: usize,
}

impl Parquet {
    pub(super) fn new(
        output_dir: PathBuf,
        compression: Compression,
        row_group_bytes: usize,
        num_threads: usize,
    ) -> Self {
        Self {
            output_dir,
            compression,
            row_group_bytes,
            num_threads,
        }
    }

    /// Generate one TPC-DS table as a Parquet file.
    pub(super) async fn generate_table(
        &self,
        table: Table,
        session: Session,
        progress: Arc<dyn ProgressTracker>,
    ) -> Result<()> {
        let path = self
            .output_dir
            .join(format!("{}.parquet", table.get_name()));

        match table {
            Table::CallCenter => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CallCenterArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::CatalogPage => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CatalogPageArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::CatalogReturns => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CatalogReturnsArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::CatalogSales => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CatalogSalesArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Customer => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CustomerArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::CustomerAddress => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CustomerAddressArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::CustomerDemographics => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    CustomerDemographicsArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::DateDim => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    DateDimArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::DbgenVersion => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    DbgenVersionArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::HouseholdDemographics => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    HouseholdDemographicsArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::IncomeBand => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    IncomeBandArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Inventory => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    InventoryArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Item => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    ItemArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Promotion => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    PromotionArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Reason => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    ReasonArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::ShipMode => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    ShipModeArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Store => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    StoreArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::StoreReturns => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    StoreReturnsArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::StoreSales => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    StoreSalesArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::TimeDim => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    TimeDimArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::Warehouse => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    WarehouseArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::WebPage => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    WebPageArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::WebReturns => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    WebReturnsArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::WebSales => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    WebSalesArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            Table::WebSite => {
                self.write_table(path, table, session, progress, |session, start, end| {
                    WebSiteArrow::new(session).with_source_row_range(start, end)
                })
                .await
            }
            _ => Ok(()),
        }
    }

    /// Write one table to a Parquet file at the specified path.
    ///
    /// `make_reader` creates a [`RecordBatchReader`] for one planned source
    /// row range; the batches of each reader are encoded (in parallel, using
    /// up to `num_threads` threads) as one row group.
    ///
    /// Progress is reported in row groups: the plan's row group count is
    /// registered, then the shared writer advances by one per written row
    /// group (the same output units as TPC-H parquet generation).
    async fn write_table<R, F>(
        &self,
        path: PathBuf,
        table: Table,
        session: Session,
        progress: Arc<dyn ProgressTracker>,
        make_reader: F,
    ) -> Result<()>
    where
        R: RecordBatchReader + Send + 'static,
        F: Fn(Session, i64, i64) -> R + 'static,
    {
        let table_name = table.get_name();
        let plan = TpcdsGenerationPlan::new(table, session.get_scaling(), self.row_group_bytes);
        progress.register(table_name, plan.row_group_count() as u64);
        let sources = plan
            .into_iter()
            .map(move |range| make_reader(session.clone(), *range.start(), *range.end()));

        // write to a temp file and then rename to avoid partial files
        let temp_path = path.with_extension("inprogress");
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);
        generate_parquet(
            writer,
            sources,
            self.num_threads,
            self.compression,
            progress,
            table_name,
        )
        .await?;
        std::fs::rename(&temp_path, &path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {temp_path:?} to {path:?} file: {err}"
            ))
        })?;

        Ok(())
    }
}
