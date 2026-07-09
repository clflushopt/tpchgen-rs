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
use std::io::Write;
use std::path::{Path, PathBuf};

use log::info;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::error::InvalidOptionError;
use tpcdsgen::output::DatWriter;
use tpcdsgen::row::{GeneratedRow, TableRow, *};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// DAT output configuration owned by the CLI layer.
#[derive(Debug, Clone)]
struct OutputOptions {
    target_directory: PathBuf,
    suffix: String,
    null_string: String,
    separator: char,
    do_not_terminate: bool,
    parallelism: i32,
    overwrite: bool,
}

impl OutputOptions {
    const DEFAULT_SUFFIX: &'static str = ".dat";
    const DEFAULT_NULL_STRING: &'static str = "";
    const DEFAULT_SEPARATOR: char = '|';
    const DEFAULT_DO_NOT_TERMINATE: bool = false;
    const DEFAULT_PARALLELISM: i32 = 1;
    const DEFAULT_OVERWRITE: bool = true;

    fn new(target_directory: PathBuf) -> Result<Self> {
        let options = Self {
            target_directory,
            suffix: Self::DEFAULT_SUFFIX.to_string(),
            null_string: Self::DEFAULT_NULL_STRING.to_string(),
            separator: Self::DEFAULT_SEPARATOR,
            do_not_terminate: Self::DEFAULT_DO_NOT_TERMINATE,
            parallelism: Self::DEFAULT_PARALLELISM,
            overwrite: Self::DEFAULT_OVERWRITE,
        };
        options.validate()?;
        Ok(options)
    }

    fn validate(&self) -> Result<()> {
        if self.target_directory.as_os_str().is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                "",
                "Directory cannot be empty",
            )
            .into());
        }

        if self.suffix.is_empty() {
            return Err(InvalidOptionError::with_message(
                "suffix",
                &self.suffix,
                "Suffix cannot be an empty string",
            )
            .into());
        }

        if self.parallelism < 1 {
            return Err(InvalidOptionError::with_message(
                "parallelism",
                &self.parallelism.to_string(),
                "Parallelism must be >= 1",
            )
            .into());
        }

        Ok(())
    }
}

/// DAT output generator.
#[derive(Debug, Clone)]
pub(super) struct Dat {
    output_options: OutputOptions,
}

impl Dat {
    pub(super) fn new(target_directory: PathBuf) -> Result<Self> {
        Ok(Self {
            output_options: OutputOptions::new(target_directory)?,
        })
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
                &self.output_options,
                progress,
            ),
            Table::CatalogPage => generate_simple::<CatalogPageRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Customer => generate_simple::<CustomerRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::CustomerAddress => generate_simple::<CustomerAddressRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::CustomerDemographics => generate_simple::<CustomerDemographicsRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::DateDim => generate_simple::<DateDimRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::DbgenVersion => generate_simple::<DbgenVersionRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::HouseholdDemographics => generate_simple::<HouseholdDemographicsRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::IncomeBand => generate_simple::<IncomeBandRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Item => {
                generate_simple::<ItemRowGenerator>(table, session, &self.output_options, progress)
            }
            Table::Promotion => generate_simple::<PromotionRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Reason => generate_simple::<ReasonRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::ShipMode => generate_simple::<ShipModeRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Store => {
                generate_simple::<StoreRowGenerator>(table, session, &self.output_options, progress)
            }
            Table::TimeDim => generate_simple::<TimeDimRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Warehouse => generate_simple::<WarehouseRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::WebPage => generate_simple::<WebPageRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::WebSite => generate_simple::<WebSiteRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),
            Table::Inventory => generate_simple::<InventoryRowGenerator>(
                table,
                session,
                &self.output_options,
                progress,
            ),

            // Sales generators write their return tables at the same time.
            Table::StoreSales => generate_store_sales(session, &self.output_options, progress),
            Table::StoreReturns => Ok(()), // Generated with StoreSales
            Table::CatalogSales => generate_catalog_sales(session, &self.output_options, progress),
            Table::CatalogReturns => Ok(()), // Generated with CatalogSales
            Table::WebSales => generate_web_sales(session, &self.output_options, progress),
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
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = G::create();
    let row_count = session.get_scaling().get_row_count(table);
    let table_name = table.get_name();
    // Register the scaling row count, then advance after each written row.
    progress.register(table_name, row_count.try_into().unwrap_or(0));

    let path = get_output_path(table, output_options);
    let file = create_output_file(&path, output_options)?;
    let mut writer = DatWriter::new(file, session.get_compat_mode());

    info!("Generating {}...", table.get_name());

    for row_number in 1..=row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            write_row(row, &mut writer, output_options)?;
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
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<StoreSalesRowGenerator>(
        Table::StoreSales,
        Table::StoreReturns,
        session,
        output_options,
        progress,
    )
}

/// Generate catalog_sales and catalog_returns together
fn generate_catalog_sales(
    session: &Session,
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<CatalogSalesRowGenerator>(
        Table::CatalogSales,
        Table::CatalogReturns,
        session,
        output_options,
        progress,
    )
}

/// Generate web_sales and web_returns together
fn generate_web_sales(
    session: &Session,
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    generate_sales_and_returns::<WebSalesRowGenerator>(
        Table::WebSales,
        Table::WebReturns,
        session,
        output_options,
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
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = G::create();
    let source_row_count = session.get_scaling().get_row_count(sales_table);
    let return_row_count = session.get_scaling().get_row_count(returns_table);
    let sales_name = sales_table.get_name();
    let returns_name = returns_table.get_name();
    // Register the scaling row counts, then advance after each written row.
    progress.register(sales_name, source_row_count.try_into().unwrap_or(0));
    progress.register(returns_name, return_row_count.try_into().unwrap_or(0));

    let sales_path = get_output_path(sales_table, output_options);
    let returns_path = get_output_path(returns_table, output_options);

    let compat_mode = session.get_compat_mode();
    let mut sales_writer = DatWriter::new(
        create_output_file(&sales_path, output_options)?,
        compat_mode,
    );
    let mut returns_writer = DatWriter::new(
        create_output_file(&returns_path, output_options)?,
        compat_mode,
    );

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
            write_row(&rows[0], &mut sales_writer, output_options)?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            write_row(&rows[1], &mut returns_writer, output_options)?;
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
fn get_output_path(table: Table, output_options: &OutputOptions) -> std::path::PathBuf {
    Path::new(&output_options.target_directory).join(format!(
        "{}{}",
        table.get_name(),
        output_options.suffix
    ))
}

fn create_output_file(path: &Path, output_options: &OutputOptions) -> Result<File> {
    if !output_options.overwrite && path.exists() {
        return Err(format!("Output file {} already exists", path.display()).into());
    }
    Ok(File::create(path)?)
}

fn write_row(
    row: &GeneratedRow,
    writer: &mut DatWriter<File>,
    output_options: &OutputOptions,
) -> Result<()> {
    if output_options.null_string.is_empty()
        && !output_options.do_not_terminate
        && output_options.separator == '|'
    {
        // Fast path: row types with a `Display` impl format the DAT line
        // with no per-field allocations; the rest go through `TableRow`.
        match row {
            GeneratedRow::StoreSales(row) => writer.write_display_row(row)?,
            GeneratedRow::StoreReturns(row) => writer.write_display_row(row)?,
            row => writer.write_table_row(row, output_options.separator)?,
        }
        return Ok(());
    }

    let buffer = writer.buffer();
    for (i, value) in row.get_values().iter().enumerate() {
        if i > 0 {
            write!(buffer, "{}", output_options.separator)?;
        }
        if value.is_empty() {
            write!(buffer, "{}", output_options.null_string)?;
        } else {
            write!(buffer, "{value}")?;
        }
    }
    if !output_options.do_not_terminate {
        write!(buffer, "{}", output_options.separator)?;
    }
    writeln!(buffer)?;
    writer.maybe_flush()?;
    Ok(())
}
