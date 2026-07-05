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

/// Generate one TPC-DS table as a CSV file in `output_dir`.
pub fn generate_table(
    table: Table,
    session: Session,
    output_dir: PathBuf,
    delimiter: char,
) -> Result<()> {
    let path = output_dir.join(format!("{}.csv", table.get_name()));

    match table {
        Table::CallCenter => write_batches(path, CallCenterArrow::new(session), delimiter),
        Table::CatalogPage => write_batches(path, CatalogPageArrow::new(session), delimiter),
        Table::CatalogReturns => write_batches(path, CatalogReturnsArrow::new(session), delimiter),
        Table::CatalogSales => write_batches(path, CatalogSalesArrow::new(session), delimiter),
        Table::Customer => write_batches(path, CustomerArrow::new(session), delimiter),
        Table::CustomerAddress => {
            write_batches(path, CustomerAddressArrow::new(session), delimiter)
        }
        Table::CustomerDemographics => {
            write_batches(path, CustomerDemographicsArrow::new(session), delimiter)
        }
        Table::DateDim => write_batches(path, DateDimArrow::new(session), delimiter),
        Table::DbgenVersion => write_batches(path, DbgenVersionArrow::new(session), delimiter),
        Table::HouseholdDemographics => {
            write_batches(path, HouseholdDemographicsArrow::new(session), delimiter)
        }
        Table::IncomeBand => write_batches(path, IncomeBandArrow::new(session), delimiter),
        Table::Inventory => write_batches(path, InventoryArrow::new(session), delimiter),
        Table::Item => write_batches(path, ItemArrow::new(session), delimiter),
        Table::Promotion => write_batches(path, PromotionArrow::new(session), delimiter),
        Table::Reason => write_batches(path, ReasonArrow::new(session), delimiter),
        Table::ShipMode => write_batches(path, ShipModeArrow::new(session), delimiter),
        Table::Store => write_batches(path, StoreArrow::new(session), delimiter),
        Table::StoreReturns => write_batches(path, StoreReturnsArrow::new(session), delimiter),
        Table::StoreSales => write_batches(path, StoreSalesArrow::new(session), delimiter),
        Table::TimeDim => write_batches(path, TimeDimArrow::new(session), delimiter),
        Table::Warehouse => write_batches(path, WarehouseArrow::new(session), delimiter),
        Table::WebPage => write_batches(path, WebPageArrow::new(session), delimiter),
        Table::WebReturns => write_batches(path, WebReturnsArrow::new(session), delimiter),
        Table::WebSales => write_batches(path, WebSalesArrow::new(session), delimiter),
        Table::WebSite => write_batches(path, WebSiteArrow::new(session), delimiter),
        _ => Ok(()),
    }
}

/// Write the record batches to a CSV file at the specified path.
fn write_batches<I>(path: PathBuf, mut batches: I, delimiter: char) -> Result<()>
where
    I: RecordBatchIterator,
{
    let temp_path = path.with_extension("inprogress");
    let file = File::create(&temp_path)
        .map_err(|err| io::Error::other(format!("Failed to create {temp_path:?}: {err}")))?;
    let writer = BufWriter::with_capacity(32 * 1024 * 1024, file);

    let mut writer = WriterBuilder::new()
        .with_header(true)
        .with_delimiter(delimiter as u8)
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
