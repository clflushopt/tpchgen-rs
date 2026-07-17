/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! TPC-DS Data Generator - Rust Implementation
//!
//! Generates TPC-DS benchmark data with byte-for-byte compatibility with the Java reference.

use crate::progress::ProgressTracker;
use std::fs::File;
use std::path::{Path, PathBuf};

use log::info;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::error::InvalidOptionError;
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output generator.
///
/// Output is always the reference DAT format: `|`-separated fields with a
/// trailing separator, one row per line, written to `<table>.dat` files via
/// each row type's `Display` impl.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    output_dir: PathBuf,
}

impl Dat {
    pub(super) fn new(output_dir: PathBuf) -> Result<Self> {
        if output_dir.as_os_str().is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                "",
                "Directory cannot be empty",
            )
            .into());
        }
        Ok(Self { output_dir })
    }

    pub(super) fn register_table(
        &self,
        table: Table,
        session: &Session,
        progress: &dyn ProgressTracker,
    ) {
        let register = |table: Table| {
            let row_count = session.get_scaling().get_row_count(table);
            progress.register(table.get_name(), row_count.try_into().unwrap_or(0));
        };

        match table {
            Table::StoreSales => {
                register(Table::StoreSales);
                register(Table::StoreReturns);
            }
            Table::CatalogSales => {
                register(Table::CatalogSales);
                register(Table::CatalogReturns);
            }
            Table::WebSales => {
                register(Table::WebSales);
                register(Table::WebReturns);
            }
            Table::StoreReturns | Table::CatalogReturns | Table::WebReturns => {}
            _ => register(table),
        }
    }

    pub(super) fn generate_table(
        &self,
        table: Table,
        session: &Session,
        progress: &dyn ProgressTracker,
    ) -> Result<()> {
        match table {
            // Simple dimension tables
            Table::CallCenter => generate_simple::<CallCenterRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::CatalogPage => generate_simple::<CatalogPageRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::Customer => {
                generate_simple::<CustomerRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::CustomerAddress => generate_simple::<CustomerAddressRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::CustomerDemographics => generate_simple::<CustomerDemographicsRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::DateDim => {
                generate_simple::<DateDimRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::DbgenVersion => generate_simple::<DbgenVersionRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::HouseholdDemographics => generate_simple::<HouseholdDemographicsRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::IncomeBand => generate_simple::<IncomeBandRowGenerator>(
                table,
                session,
                &self.output_dir,
                progress,
            ),
            Table::Item => {
                generate_simple::<ItemRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::Promotion => {
                generate_simple::<PromotionRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::Reason => {
                generate_simple::<ReasonRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::ShipMode => {
                generate_simple::<ShipModeRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::Store => {
                generate_simple::<StoreRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::TimeDim => {
                generate_simple::<TimeDimRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::Warehouse => {
                generate_simple::<WarehouseRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::WebPage => {
                generate_simple::<WebPageRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::WebSite => {
                generate_simple::<WebSiteRowGenerator>(table, session, &self.output_dir, progress)
            }
            Table::Inventory => {
                generate_simple::<InventoryRowGenerator>(table, session, &self.output_dir, progress)
            }

            // Sales generators write their return tables at the same time.
            Table::StoreSales => generate_store_sales(session, &self.output_dir, progress),
            Table::StoreReturns => Ok(()), // Generated with StoreSales
            Table::CatalogSales => generate_catalog_sales(session, &self.output_dir, progress),
            Table::CatalogReturns => Ok(()), // Generated with CatalogSales
            Table::WebSales => generate_web_sales(session, &self.output_dir, progress),
            Table::WebReturns => Ok(()), // Generated with WebSales

            // Source tables - skip
            _ => Ok(()),
        }
    }
}

/// Trait for creating row generators
trait RowGeneratorFactory: RowGenerator + Sized {
    fn create() -> Self;
}

macro_rules! impl_factory {
    ($($gen:ty),*) => {
        $(
            impl RowGeneratorFactory for $gen {
                fn create() -> Self { Self::new() }
            }
        )*
    };
}

// Implement factory for all simple generators
impl_factory!(
    CallCenterRowGenerator,
    CatalogPageRowGenerator,
    CustomerRowGenerator,
    CustomerAddressRowGenerator,
    CustomerDemographicsRowGenerator,
    DateDimRowGenerator,
    DbgenVersionRowGenerator,
    HouseholdDemographicsRowGenerator,
    IncomeBandRowGenerator,
    InventoryRowGenerator,
    ItemRowGenerator,
    PromotionRowGenerator,
    ReasonRowGenerator,
    ShipModeRowGenerator,
    StoreRowGenerator,
    TimeDimRowGenerator,
    WarehouseRowGenerator,
    WebPageRowGenerator,
    WebSiteRowGenerator
);

// Implement factory for generators that emit sales and returns rows
impl_factory!(
    CatalogSalesRowGenerator,
    StoreSalesRowGenerator,
    WebSalesRowGenerator
);

/// Generate a simple table (one row per row_number, no child tables)
fn generate_simple<G: RowGeneratorFactory>(
    table: Table,
    session: &Session,
    output_dir: &Path,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = G::create();
    let row_count = session.get_scaling().get_row_count(table);
    let table_name = table.get_name();

    let path = get_output_path(table, output_dir);
    let file = File::create(&path)?;
    let mut writer = DatWriter::new(file, session.get_compat_mode());

    info!("Generating {}...", table.get_name());

    for row_number in 1..=row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            writer.write_display_row(row)?;
        }

        generator.consume_remaining_seeds_for_row();
        progress.increment(table_name, 1);
    }

    writer.flush()?;
    info!(
        "Generated {}: {} rows -> {}",
        table.get_name(),
        row_count,
        path.display()
    );

    Ok(())
}

/// Generate store_sales and store_returns together
fn generate_store_sales(
    session: &Session,
    output_dir: &Path,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<StoreSalesRowGenerator>(
        Table::StoreSales,
        Table::StoreReturns,
        session,
        output_dir,
        progress,
    )
}

/// Generate catalog_sales and catalog_returns together
fn generate_catalog_sales(
    session: &Session,
    output_dir: &Path,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<CatalogSalesRowGenerator>(
        Table::CatalogSales,
        Table::CatalogReturns,
        session,
        output_dir,
        progress,
    )
}

/// Generate web_sales and web_returns together
fn generate_web_sales(
    session: &Session,
    output_dir: &Path,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<WebSalesRowGenerator>(
        Table::WebSales,
        Table::WebReturns,
        session,
        output_dir,
        progress,
    )
}

/// Generate sales and returns tables in one pass.
///
/// Sales generators can emit rows for both output tables. Keeping them paired
/// preserves row advancement and seed consumption.
fn generate_sales_and_returns<G: RowGeneratorFactory>(
    sales_table: Table,
    returns_table: Table,
    session: &Session,
    output_dir: &Path,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = G::create();
    let source_row_count = session.get_scaling().get_row_count(sales_table);
    let sales_name = sales_table.get_name();
    let returns_name = returns_table.get_name();

    let sales_path = get_output_path(sales_table, output_dir);
    let returns_path = get_output_path(returns_table, output_dir);

    let compat_mode = session.get_compat_mode();
    let mut sales_writer = DatWriter::new(File::create(&sales_path)?, compat_mode);
    let mut returns_writer = DatWriter::new(File::create(&returns_path)?, compat_mode);

    info!(
        "Generating {} + {}...",
        sales_table.get_name(),
        returns_table.get_name()
    );

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= source_row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            sales_writer.write_display_row(&rows[0])?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            returns_writer.write_display_row(&rows[1])?;
            returns_count += 1;
            progress.increment(returns_name, 1);
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            row_number += 1;
            progress.increment(sales_name, 1);
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;

    info!(
        "Generated {} + {}: {} sales, {} returns -> {}, {}",
        sales_table.get_name(),
        returns_table.get_name(),
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Get output file path for a table
fn get_output_path(table: Table, output_dir: &Path) -> PathBuf {
    output_dir.join(format!("{}.dat", table.get_name()))
}
