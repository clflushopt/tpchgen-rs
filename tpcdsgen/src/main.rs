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

use clap::Parser;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use tpcdsgen::config::{Options, Session, Table};
use tpcdsgen::output::Iso8859Writer;
use tpcdsgen::row::*;
use tpcdsgen::types::Date;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let options = Options::parse();
    let session = options.to_session()?;

    println!("TPC-DS Data Generator (Rust)");
    println!("Scale factor: {}", session.get_scaling().get_scale());
    println!("Output directory: {}", session.get_target_directory());

    let start = Instant::now();

    if session.generate_only_one_table() {
        let table = session.get_only_table_to_generate();
        generate_table(table, &session)?;
    } else {
        // Generate all main tables
        for table in Table::main_tables() {
            generate_table(table, &session)?;
        }
    }

    let elapsed = start.elapsed();
    println!("\nCompleted in {:.2}s", elapsed.as_secs_f64());

    Ok(())
}

fn generate_table(table: Table, session: &Session) -> Result<()> {
    match table {
        // Simple dimension tables
        Table::CallCenter => generate_simple::<CallCenterRowGenerator>(table, session),
        Table::CatalogPage => generate_simple::<CatalogPageRowGenerator>(table, session),
        Table::Customer => generate_simple::<CustomerRowGenerator>(table, session),
        Table::CustomerAddress => generate_simple::<CustomerAddressRowGenerator>(table, session),
        Table::CustomerDemographics => {
            generate_simple::<CustomerDemographicsRowGenerator>(table, session)
        }
        Table::DateDim => generate_simple::<DateDimRowGenerator>(table, session),
        Table::DbgenVersion => generate_simple::<DbgenVersionRowGenerator>(table, session),
        Table::HouseholdDemographics => {
            generate_simple::<HouseholdDemographicsRowGenerator>(table, session)
        }
        Table::IncomeBand => generate_simple::<IncomeBandRowGenerator>(table, session),
        Table::Item => generate_simple::<ItemRowGenerator>(table, session),
        Table::Promotion => generate_simple::<PromotionRowGenerator>(table, session),
        Table::Reason => generate_simple::<ReasonRowGenerator>(table, session),
        Table::ShipMode => generate_simple::<ShipModeRowGenerator>(table, session),
        Table::Store => generate_simple::<StoreRowGenerator>(table, session),
        Table::TimeDim => generate_simple::<TimeDimRowGenerator>(table, session),
        Table::Warehouse => generate_simple::<WarehouseRowGenerator>(table, session),
        Table::WebPage => generate_simple::<WebPageRowGenerator>(table, session),
        Table::WebSite => generate_simple::<WebSiteRowGenerator>(table, session),

        // Sales + Returns pairs
        Table::StoreSales => generate_store_sales(session),
        Table::StoreReturns => Ok(()), // Generated with StoreSales
        Table::CatalogSales => generate_catalog_sales(session),
        Table::CatalogReturns => Ok(()), // Generated with CatalogSales
        Table::WebSales => generate_web_sales(session),
        Table::WebReturns => Ok(()), // Generated with WebSales

        // Special tables
        Table::Inventory => generate_inventory(session),

        // Source tables - skip
        _ => Ok(()),
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

/// Generate a simple table (one row per row_number, no child tables)
fn generate_simple<G: RowGeneratorFactory>(table: Table, session: &Session) -> Result<()> {
    let mut generator = G::create();
    let row_count = session.get_scaling().get_row_count(table);

    let path = get_output_path(table, session);
    let file = File::create(&path)?;
    let mut writer = Iso8859Writer::new(BufWriter::new(file));

    print!("Generating {}... ", table.get_name());
    std::io::stdout().flush()?;

    for row_number in 1..=row_count {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            row.write_to(&mut writer, session.get_separator())?;
        }

        generator.consume_remaining_seeds_for_row();
    }

    writer.flush()?;
    println!("{} rows -> {}", row_count, path.display());

    Ok(())
}

/// Generate store_sales and store_returns together
fn generate_store_sales(session: &Session) -> Result<()> {
    let mut generator = StoreSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::StoreSales);

    let sales_path = get_output_path(Table::StoreSales, session);
    let returns_path = get_output_path(Table::StoreReturns, session);

    let mut sales_writer = Iso8859Writer::new(BufWriter::new(File::create(&sales_path)?));
    let mut returns_writer = Iso8859Writer::new(BufWriter::new(File::create(&returns_path)?));

    print!("Generating store_sales + store_returns... ");
    std::io::stdout().flush()?;

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            rows[0].write_to(&mut sales_writer, session.get_separator())?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            rows[1].write_to(&mut returns_writer, session.get_separator())?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            generator.consume_child_seeds();
            row_number += 1;
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;

    println!(
        "{} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate catalog_sales and catalog_returns together
fn generate_catalog_sales(session: &Session) -> Result<()> {
    let mut generator = CatalogSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::CatalogSales);

    let sales_path = get_output_path(Table::CatalogSales, session);
    let returns_path = get_output_path(Table::CatalogReturns, session);

    let mut sales_writer = Iso8859Writer::new(BufWriter::new(File::create(&sales_path)?));
    let mut returns_writer = Iso8859Writer::new(BufWriter::new(File::create(&returns_path)?));

    print!("Generating catalog_sales + catalog_returns... ");
    std::io::stdout().flush()?;

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            rows[0].write_to(&mut sales_writer, session.get_separator())?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            rows[1].write_to(&mut returns_writer, session.get_separator())?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            generator.consume_child_seeds();
            row_number += 1;
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;

    println!(
        "{} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate web_sales and web_returns together
fn generate_web_sales(session: &Session) -> Result<()> {
    let mut generator = WebSalesRowGenerator::new();
    let num_orders = session.get_scaling().get_row_count(Table::WebSales);

    let sales_path = get_output_path(Table::WebSales, session);
    let returns_path = get_output_path(Table::WebReturns, session);

    let mut sales_writer = Iso8859Writer::new(BufWriter::new(File::create(&sales_path)?));
    let mut returns_writer = Iso8859Writer::new(BufWriter::new(File::create(&returns_path)?));

    print!("Generating web_sales + web_returns... ");
    std::io::stdout().flush()?;

    let mut sales_count = 0i64;
    let mut returns_count = 0i64;
    let mut row_number = 1i64;

    while row_number <= num_orders {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;
        let rows = result.get_rows();

        if !rows.is_empty() {
            rows[0].write_to(&mut sales_writer, session.get_separator())?;
            sales_count += 1;
        }

        if rows.len() > 1 {
            rows[1].write_to(&mut returns_writer, session.get_separator())?;
            returns_count += 1;
        }

        if result.should_end_row() {
            generator.consume_remaining_seeds_for_row();
            generator.consume_child_seeds();
            row_number += 1;
        }
    }

    sales_writer.flush()?;
    returns_writer.flush()?;

    println!(
        "{} sales, {} returns -> {}, {}",
        sales_count,
        returns_count,
        sales_path.display(),
        returns_path.display()
    );

    Ok(())
}

/// Generate inventory table (special row count calculation)
fn generate_inventory(session: &Session) -> Result<()> {
    let mut generator = InventoryRowGenerator::new();
    let scaling = session.get_scaling();

    let item_count = scaling.get_id_count(tpcdsgen::config::Table::Item);
    let warehouse_count = scaling.get_row_count(tpcdsgen::config::Table::Warehouse);
    let n_days = Date::JULIAN_DATE_MAXIMUM - Date::JULIAN_DATE_MINIMUM;
    let n_weeks = (n_days + 7) / 7;
    let num_rows = item_count * warehouse_count * n_weeks as i64;

    let path = get_output_path(Table::Inventory, session);
    let mut writer = Iso8859Writer::new(BufWriter::new(File::create(&path)?));

    print!("Generating inventory... ");
    std::io::stdout().flush()?;

    for row_number in 1..=num_rows {
        let result = generator.generate_row_and_child_rows(row_number, session, None, None)?;

        for row in result.get_rows() {
            row.write_to(&mut writer, session.get_separator())?;
        }

        generator.consume_remaining_seeds_for_row();
    }

    writer.flush()?;
    println!("{} rows -> {}", num_rows, path.display());

    Ok(())
}

/// Get output file path for a table
fn get_output_path(table: Table, session: &Session) -> std::path::PathBuf {
    Path::new(session.get_target_directory()).join(format!(
        "{}{}",
        table.get_name(),
        session.get_suffix()
    ))
}
