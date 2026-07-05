//! TPC-DS CSV output.

use arrow::array::RecordBatch;
use arrow_csv::writer::WriterBuilder;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tpcdsgen::config::{Session, Table};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CatalogReturnsArrow, CatalogSalesArrow,
    CustomerAddressArrow, CustomerArrow, CustomerDemographicsArrow, DateDimArrow,
    DbgenVersionArrow, HouseholdDemographicsArrow, IncomeBandArrow, InventoryArrow, ItemArrow,
    PromotionArrow, ReasonArrow, RecordBatchIterator, ShipModeArrow, StoreArrow, StoreReturnsArrow,
    StoreSalesArrow, TimeDimArrow, WarehouseArrow, WebPageArrow, WebReturnsArrow, WebSalesArrow,
    WebSiteArrow,
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
    pub(super) fn generate_table(&self, table: Table, session: Session) -> Result<()> {
        let path = self.output_dir.join(format!("{}.csv", table.get_name()));

        match table {
            Table::CallCenter => self.write_batches(path, CallCenterArrow::new(session)),
            Table::CatalogPage => self.write_batches(path, CatalogPageArrow::new(session)),
            Table::CatalogReturns => self.write_batches(path, CatalogReturnsArrow::new(session)),
            Table::CatalogSales => self.write_batches(path, CatalogSalesArrow::new(session)),
            Table::Customer => self.write_batches(path, CustomerArrow::new(session)),
            Table::CustomerAddress => self.write_batches(path, CustomerAddressArrow::new(session)),
            Table::CustomerDemographics => {
                self.write_batches(path, CustomerDemographicsArrow::new(session))
            }
            Table::DateDim => self.write_batches(path, DateDimArrow::new(session)),
            Table::DbgenVersion => self.write_batches(path, DbgenVersionArrow::new(session)),
            Table::HouseholdDemographics => {
                self.write_batches(path, HouseholdDemographicsArrow::new(session))
            }
            Table::IncomeBand => self.write_batches(path, IncomeBandArrow::new(session)),
            Table::Inventory => self.write_batches(path, InventoryArrow::new(session)),
            Table::Item => self.write_batches(path, ItemArrow::new(session)),
            Table::Promotion => self.write_batches(path, PromotionArrow::new(session)),
            Table::Reason => self.write_batches(path, ReasonArrow::new(session)),
            Table::ShipMode => self.write_batches(path, ShipModeArrow::new(session)),
            Table::Store => self.write_batches(path, StoreArrow::new(session)),
            Table::StoreReturns => self.write_batches(path, StoreReturnsArrow::new(session)),
            Table::StoreSales => self.write_batches(path, StoreSalesArrow::new(session)),
            Table::TimeDim => self.write_batches(path, TimeDimArrow::new(session)),
            Table::Warehouse => self.write_batches(path, WarehouseArrow::new(session)),
            Table::WebPage => self.write_batches(path, WebPageArrow::new(session)),
            Table::WebReturns => self.write_batches(path, WebReturnsArrow::new(session)),
            Table::WebSales => self.write_batches(path, WebSalesArrow::new(session)),
            Table::WebSite => self.write_batches(path, WebSiteArrow::new(session)),
            _ => Ok(()),
        }
    }

    /// Write the record batches to a CSV file at the specified path.
    fn write_batches<I>(&self, path: PathBuf, mut batches: I) -> Result<()>
    where
        I: RecordBatchIterator,
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
        writer.write(&RecordBatch::new_empty(Arc::clone(batches.schema())))?;

        for batch in &mut batches {
            writer.write(&batch)?;
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
