//! TPC-DS Parquet output.

use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::io::{self, BufWriter};
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

/// Parquet output generator.
#[derive(Debug, Clone)]
pub(super) struct Parquet {
    output_dir: PathBuf,
    compression: Compression,
}

impl Parquet {
    pub(super) fn new(output_dir: PathBuf, compression: Compression) -> Self {
        Self {
            output_dir,
            compression,
        }
    }

    /// Generate one TPC-DS table as a Parquet file.
    pub(super) fn generate_table(&self, table: Table, session: Session) -> Result<()> {
        let path = self
            .output_dir
            .join(format!("{}.parquet", table.get_name()));

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

    /// Write the record batches to a Parquet file at the specified path.
    fn write_batches<I>(&self, path: PathBuf, mut batches: I) -> Result<()>
    where
        I: RecordBatchIterator,
    {
        let temp_path = path.with_extension("inprogress");
        let file = File::create(&temp_path)
            .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
        let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);
        let writer_properties = WriterProperties::builder()
            .set_compression(self.compression)
            .build();
        let mut writer = ArrowWriter::try_new(
            writer,
            Arc::clone(batches.schema()),
            Some(writer_properties),
        )?;

        for batch in &mut batches {
            writer.write(&batch)?;
        }

        writer.close()?;
        std::fs::rename(&temp_path, &path).map_err(|err| {
            io::Error::other(format!(
                "Failed to rename {temp_path:?} to {path:?} file: {err}"
            ))
        })?;

        Ok(())
    }
}
