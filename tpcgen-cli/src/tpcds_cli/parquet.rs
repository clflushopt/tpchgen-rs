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

/// Generate one TPC-DS table as a Parquet file in `output_dir`.
pub fn generate_table(
    table: Table,
    session: Session,
    output_dir: PathBuf,
    compression: Compression,
) -> Result<()> {
    let path = output_dir.join(format!("{}.parquet", table.get_name()));

    match table {
        Table::CallCenter => write_batches(path, CallCenterArrow::new(session), compression),
        Table::CatalogPage => write_batches(path, CatalogPageArrow::new(session), compression),
        Table::CatalogReturns => {
            write_batches(path, CatalogReturnsArrow::new(session), compression)
        }
        Table::CatalogSales => write_batches(path, CatalogSalesArrow::new(session), compression),
        Table::Customer => write_batches(path, CustomerArrow::new(session), compression),
        Table::CustomerAddress => {
            write_batches(path, CustomerAddressArrow::new(session), compression)
        }
        Table::CustomerDemographics => {
            write_batches(path, CustomerDemographicsArrow::new(session), compression)
        }
        Table::DateDim => write_batches(path, DateDimArrow::new(session), compression),
        Table::DbgenVersion => write_batches(path, DbgenVersionArrow::new(session), compression),
        Table::HouseholdDemographics => {
            write_batches(path, HouseholdDemographicsArrow::new(session), compression)
        }
        Table::IncomeBand => write_batches(path, IncomeBandArrow::new(session), compression),
        Table::Inventory => write_batches(path, InventoryArrow::new(session), compression),
        Table::Item => write_batches(path, ItemArrow::new(session), compression),
        Table::Promotion => write_batches(path, PromotionArrow::new(session), compression),
        Table::Reason => write_batches(path, ReasonArrow::new(session), compression),
        Table::ShipMode => write_batches(path, ShipModeArrow::new(session), compression),
        Table::Store => write_batches(path, StoreArrow::new(session), compression),
        Table::StoreReturns => write_batches(path, StoreReturnsArrow::new(session), compression),
        Table::StoreSales => write_batches(path, StoreSalesArrow::new(session), compression),
        Table::TimeDim => write_batches(path, TimeDimArrow::new(session), compression),
        Table::Warehouse => write_batches(path, WarehouseArrow::new(session), compression),
        Table::WebPage => write_batches(path, WebPageArrow::new(session), compression),
        Table::WebReturns => write_batches(path, WebReturnsArrow::new(session), compression),
        Table::WebSales => write_batches(path, WebSalesArrow::new(session), compression),
        Table::WebSite => write_batches(path, WebSiteArrow::new(session), compression),
        _ => Ok(()),
    }
}

/// Write the record batches to a Parquet file at the specified path with the given compression.
fn write_batches<I>(path: PathBuf, mut batches: I, compression: Compression) -> Result<()>
where
    I: RecordBatchIterator,
{
    let temp_path = path.with_extension("inprogress");
    let file = File::create(&temp_path)
        .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
    let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);
    let writer_properties = WriterProperties::builder()
        .set_compression(compression)
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
