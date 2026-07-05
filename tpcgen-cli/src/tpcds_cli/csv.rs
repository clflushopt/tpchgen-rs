//! TPC-DS CSV output.

use crate::tpch_cli::progress::ProgressTracker;
use arrow::array::RecordBatch;
use arrow::record_batch::RecordBatchReader;
use arrow_csv::writer::WriterBuilder;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use tpcdsgen::config::{Session, Table};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, ShipModeArrow, StoreArrow, StoreReturnsArrow, StoreSalesArrow,
    TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow, WebSiteArrow,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// CSV output generator.
#[derive(Debug, Clone)]
pub(super) struct Csv {
    output_dir: PathBuf,
    delimiter: char,
}

impl Csv {
    pub(super) fn new(output_dir: PathBuf, delimiter: char) -> Self {
        Self {
            output_dir,
            delimiter,
        }
    }

    /// Generate one TPC-DS table as a CSV file.
    pub(super) fn generate_table(
        &self,
        table: Table,
        session: Session,
        progress: &dyn ProgressTracker,
    ) -> Result<()> {
        let path = self.output_dir.join(format!("{}.csv", table.get_name()));
        let table_name = table.get_name();
        let rows: u64 = session
            .get_scaling()
            .get_row_count(table)
            .try_into()
            .unwrap_or(0);
        // Register the table row count, then advance by each written batch's
        // row count.
        progress.register(table_name, rows);

        match table {
            Table::CallCenter => {
                self.write_batches(path, CallCenterArrow::new(session), progress, table_name)
            }
            Table::CatalogPage => {
                self.write_batches(path, CatalogPageArrow::new(session), progress, table_name)
            }
            Table::CatalogReturns => self.write_batches(
                path,
                CatalogReturnsArrow::new(session),
                progress,
                table_name,
            ),
            Table::CatalogSales => {
                self.write_batches(path, CatalogSalesArrow::new(session), progress, table_name)
            }
            Table::Customer => {
                self.write_batches(path, CustomerArrow::new(session), progress, table_name)
            }
            Table::CustomerAddress => self.write_batches(
                path,
                CustomerAddressArrow::new(session),
                progress,
                table_name,
            ),
            Table::CustomerDemographics => self.write_batches(
                path,
                CustomerDemographicsArrow::new(session),
                progress,
                table_name,
            ),
            Table::DateDim => {
                self.write_batches(path, DateDimArrow::new(session), progress, table_name)
            }
            Table::DbgenVersion => {
                self.write_batches(path, DbgenVersionArrow::new(session), progress, table_name)
            }
            Table::HouseholdDemographics => self.write_batches(
                path,
                HouseholdDemographicsArrow::new(session),
                progress,
                table_name,
            ),
            Table::IncomeBand => {
                self.write_batches(path, IncomeBandArrow::new(session), progress, table_name)
            }
            Table::Inventory => {
                self.write_batches(path, InventoryArrow::new(session), progress, table_name)
            }
            Table::Item => self.write_batches(path, ItemArrow::new(session), progress, table_name),
            Table::Promotion => {
                self.write_batches(path, PromotionArrow::new(session), progress, table_name)
            }
            Table::Reason => {
                self.write_batches(path, ReasonArrow::new(session), progress, table_name)
            }
            Table::ShipMode => {
                self.write_batches(path, ShipModeArrow::new(session), progress, table_name)
            }
            Table::Store => {
                self.write_batches(path, StoreArrow::new(session), progress, table_name)
            }
            Table::StoreReturns => {
                self.write_batches(path, StoreReturnsArrow::new(session), progress, table_name)
            }
            Table::StoreSales => {
                self.write_batches(path, StoreSalesArrow::new(session), progress, table_name)
            }
            Table::TimeDim => {
                self.write_batches(path, TimeDimArrow::new(session), progress, table_name)
            }
            Table::Warehouse => {
                self.write_batches(path, WarehouseArrow::new(session), progress, table_name)
            }
            Table::WebPage => {
                self.write_batches(path, WebPageArrow::new(session), progress, table_name)
            }
            Table::WebReturns => {
                self.write_batches(path, WebReturnsArrow::new(session), progress, table_name)
            }
            Table::WebSales => {
                self.write_batches(path, WebSalesArrow::new(session), progress, table_name)
            }
            Table::WebSite => {
                self.write_batches(path, WebSiteArrow::new(session), progress, table_name)
            }
            _ => Ok(()),
        }
    }

    /// Write the record batches to a CSV file at the specified path.
    fn write_batches<I>(
        &self,
        path: PathBuf,
        mut batches: I,
        progress: &dyn ProgressTracker,
        table_name: &'static str,
    ) -> Result<()>
    where
        I: RecordBatchReader,
    {
        let temp_path = path.with_extension("inprogress");
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);

        let mut writer = WriterBuilder::new()
            .with_header(true)
            .with_delimiter(self.delimiter as u8)
            .build(writer);

        // Write the header first.
        writer.write(&RecordBatch::new_empty(batches.schema()))?;

        for batch in &mut batches {
            let batch = batch?;
            writer.write(&batch)?;
            progress.increment(table_name, batch.num_rows() as u64);
        }

        let mut writer = writer.into_inner();
        writer.flush()?;

        std::fs::rename(&temp_path, &path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {temp_path:?} to {path:?} file: {err}"
            ))
        })?;

        Ok(())
    }
}
