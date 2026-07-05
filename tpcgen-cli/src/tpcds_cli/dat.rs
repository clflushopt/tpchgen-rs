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

use crate::tpch_cli::progress::ProgressTracker;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use log::info;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::error::InvalidOptionError;
use tpcdsgen::output::CompatWriter;
use tpcdsgen::row::{GeneratedRow, TableRow, *};
use tpcdsgen::types::Date;

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

            // Sales generators write their return tables at the same time.
            Table::StoreSales => generate_store_sales(session, &self.output_options, progress),
            Table::StoreReturns => Ok(()), // Generated with StoreSales
            Table::CatalogSales => generate_catalog_sales(session, &self.output_options, progress),
            Table::CatalogReturns => Ok(()), // Generated with CatalogSales
            Table::WebSales => generate_web_sales(session, &self.output_options, progress),
            Table::WebReturns => Ok(()), // Generated with WebSales

            // Special tables
            Table::Inventory => generate_inventory(session, &self.output_options, progress),

            // Source tables - skip
            _ => Ok(()),
        }
    }
}

/// Trait for creating row generators
trait RowGeneratorFactory: RowGenerator + Sized {
    fn create() -> Self;
}

// Implement factory for all simple generators
macro_rules! impl_factory {
    ($($gen:ty),*) => {
        $(
            impl RowGeneratorFactory for $gen {
                fn create() -> Self { Self::new() }
            }
        )*
    };
}

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

/// Generate a simple table (one row per row_number, no paired return output)
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
    progress.register(table_name, progress_units(row_count));

    let path = get_output_path(table, output_options);
    let file = create_output_file(&path, output_options)?;
    let mut writer = CompatWriter::new(BufWriter::new(file), session.get_compat_mode());

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
    let mut generator = StoreSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::StoreSales);
    let sales_name = Table::StoreSales.get_name();
    let returns_name = Table::StoreReturns.get_name();
    // Return rows are produced by the sales generator; the return bar completes
    // after the paired files are written.
    progress.register(sales_name, progress_units(num_orders));
    progress.register(returns_name, 1);

    let sales_path = get_output_path(Table::StoreSales, output_options);
    let returns_path = get_output_path(Table::StoreReturns, output_options);

    let compat_mode = session.get_compat_mode();
    let mut sales_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&sales_path, output_options)?),
        compat_mode,
    );
    let mut returns_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&returns_path, output_options)?),
        compat_mode,
    );

    info!("Generating store_sales + store_returns...");

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            write_row(&rows[0], &mut sales_writer, output_options)?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            write_row(&rows[1], &mut returns_writer, output_options)?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            row_number += 1;
            progress.increment(sales_name, 1);
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;
    progress.increment(returns_name, 1);

    info!(
        "Generated store_sales + store_returns: {} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate catalog_sales and catalog_returns together
fn generate_catalog_sales(
    session: &Session,
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = CatalogSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::CatalogSales);
    let sales_name = Table::CatalogSales.get_name();
    let returns_name = Table::CatalogReturns.get_name();
    // Return rows are produced by the sales generator; the return bar completes
    // after the paired files are written.
    progress.register(sales_name, progress_units(num_orders));
    progress.register(returns_name, 1);

    let sales_path = get_output_path(Table::CatalogSales, output_options);
    let returns_path = get_output_path(Table::CatalogReturns, output_options);

    let compat_mode = session.get_compat_mode();
    let mut sales_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&sales_path, output_options)?),
        compat_mode,
    );
    let mut returns_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&returns_path, output_options)?),
        compat_mode,
    );

    info!("Generating catalog_sales + catalog_returns...");

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            write_row(&rows[0], &mut sales_writer, output_options)?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            write_row(&rows[1], &mut returns_writer, output_options)?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            row_number += 1;
            progress.increment(sales_name, 1);
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;
    progress.increment(returns_name, 1);

    info!(
        "Generated catalog_sales + catalog_returns: {} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate web_sales and web_returns together
fn generate_web_sales(
    session: &Session,
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = WebSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::WebSales);
    let sales_name = Table::WebSales.get_name();
    let returns_name = Table::WebReturns.get_name();
    // Return rows are produced by the sales generator; the return bar completes
    // after the paired files are written.
    progress.register(sales_name, progress_units(num_orders));
    progress.register(returns_name, 1);

    let sales_path = get_output_path(Table::WebSales, output_options);
    let returns_path = get_output_path(Table::WebReturns, output_options);

    let compat_mode = session.get_compat_mode();
    let mut sales_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&sales_path, output_options)?),
        compat_mode,
    );
    let mut returns_writer = CompatWriter::new(
        BufWriter::new(create_output_file(&returns_path, output_options)?),
        compat_mode,
    );

    info!("Generating web_sales + web_returns...");

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            write_row(&rows[0], &mut sales_writer, output_options)?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            write_row(&rows[1], &mut returns_writer, output_options)?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            row_number += 1;
            progress.increment(sales_name, 1);
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;
    progress.increment(returns_name, 1);

    info!(
        "Generated web_sales + web_returns: {} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate inventory table (special row count calculation)
fn generate_inventory(
    session: &Session,
    output_options: &OutputOptions,
    progress: &dyn ProgressTracker,
) -> Result<()> {
    let mut generator = InventoryRowGenerator::new();
    let scaling = session.get_scaling();

    let item_count = scaling.get_id_count(Table::Item);
    let warehouse_count = scaling.get_row_count(Table::Warehouse);
    let n_days = Date::JULIAN_DATE_MAXIMUM - Date::JULIAN_DATE_MINIMUM;
    let n_weeks = (n_days + 7) / 7;
    let num_rows = item_count * warehouse_count * n_weeks as i64;
    let table_name = Table::Inventory.get_name();
    // Register the calculated row count, then advance after each written row.
    progress.register(table_name, progress_units(num_rows));

    let path = get_output_path(Table::Inventory, output_options);
    let mut writer = CompatWriter::new(
        BufWriter::new(create_output_file(&path, output_options)?),
        session.get_compat_mode(),
    );

    info!("Generating inventory...");

    for row_number in 1..=num_rows {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            write_row(row, &mut writer, output_options)?;
        }

        generator.consume_remaining_seeds_for_row();
        progress.increment(table_name, 1);
    }

    writer.flush()?;
    info!(
        "Generated inventory: {} rows -> {}",
        num_rows,
        path.display()
    );

    Ok(())
}

fn progress_units(count: i64) -> u64 {
    count.try_into().unwrap_or(0)
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
    writer: &mut dyn Write,
    output_options: &OutputOptions,
) -> Result<()> {
    if output_options.null_string.is_empty() && !output_options.do_not_terminate {
        row.write_to(writer, output_options.separator)?;
        return Ok(());
    }

    for (i, value) in row.get_values().iter().enumerate() {
        if i > 0 {
            write!(writer, "{}", output_options.separator)?;
        }
        if value.is_empty() {
            write!(writer, "{}", output_options.null_string)?;
        } else {
            write!(writer, "{value}")?;
        }
    }
    if !output_options.do_not_terminate {
        write!(writer, "{}", output_options.separator)?;
    }
    writeln!(writer)?;
    Ok(())
}
