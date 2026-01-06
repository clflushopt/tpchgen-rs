/*
 * Benchmark binary for timing TPC-DS table generation at various scale factors.
 *
 * Usage: benchmark --scale <SCALE> --table <TABLE> [--output-dir <DIR>]
 */

use clap::Parser;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tpcdsgen::config::{Session, Table};
use tpcdsgen::output::Iso8859Writer;
use tpcdsgen::row::*;

#[derive(Parser, Debug)]
#[command(name = "benchmark")]
#[command(about = "Benchmark TPC-DS table generation")]
struct Args {
    /// Scale factor (1, 10, 100, etc.)
    #[arg(short, long, default_value = "1")]
    scale: f64,

    /// Table to generate (or "all" for all tables)
    #[arg(short, long, default_value = "all")]
    table: String,

    /// Output directory (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Suppress output files (measure generation speed only)
    #[arg(long, default_value = "false")]
    no_output: bool,

    /// Output results as JSON
    #[arg(long, default_value = "false")]
    json: bool,
}

fn create_session(scale: f64) -> Session {
    Session::new(
        scale,
        ".".to_string(),
        ".dat".to_string(),
        None,
        "".to_string(),
        '|',
        false,
        false,
        1,
        true,
    )
}

struct BenchmarkResult {
    table: String,
    _scale: f64,
    rows: i64,
    duration_ms: u128,
    rows_per_sec: f64,
}

fn benchmark_table(
    table_name: &str,
    session: &Session,
    output_dir: &Path,
    no_output: bool,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut row_count: i64 = 0;

    match table_name {
        "call_center" => {
            let mut gen = CallCenterRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::CallCenter);
            if !no_output {
                let file = File::create(output_dir.join("call_center.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "catalog_page" => {
            let mut gen = CatalogPageRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::CatalogPage);
            if !no_output {
                let file = File::create(output_dir.join("catalog_page.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "customer" => {
            let mut gen = CustomerRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Customer);
            if !no_output {
                let file = File::create(output_dir.join("customer.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "customer_address" => {
            let mut gen = CustomerAddressRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::CustomerAddress);
            if !no_output {
                let file = File::create(output_dir.join("customer_address.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "customer_demographics" => {
            let mut gen = CustomerDemographicsRowGenerator::new();
            let num_rows = session
                .get_scaling()
                .get_row_count(Table::CustomerDemographics);
            if !no_output {
                let file = File::create(output_dir.join("customer_demographics.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "date_dim" => {
            let mut gen = DateDimRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::DateDim);
            if !no_output {
                let file = File::create(output_dir.join("date_dim.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "household_demographics" => {
            let mut gen = HouseholdDemographicsRowGenerator::new();
            let num_rows = session
                .get_scaling()
                .get_row_count(Table::HouseholdDemographics);
            if !no_output {
                let file = File::create(output_dir.join("household_demographics.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "income_band" => {
            let mut gen = IncomeBandRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::IncomeBand);
            if !no_output {
                let file = File::create(output_dir.join("income_band.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "item" => {
            let mut gen = ItemRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Item);
            if !no_output {
                let file = File::create(output_dir.join("item.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "promotion" => {
            let mut gen = PromotionRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Promotion);
            if !no_output {
                let file = File::create(output_dir.join("promotion.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "reason" => {
            let mut gen = ReasonRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Reason);
            if !no_output {
                let file = File::create(output_dir.join("reason.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "ship_mode" => {
            let mut gen = ShipModeRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::ShipMode);
            if !no_output {
                let file = File::create(output_dir.join("ship_mode.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "store" => {
            let mut gen = StoreRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Store);
            if !no_output {
                let file = File::create(output_dir.join("store.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "time_dim" => {
            let mut gen = TimeDimRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::TimeDim);
            if !no_output {
                let file = File::create(output_dir.join("time_dim.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "warehouse" => {
            let mut gen = WarehouseRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Warehouse);
            if !no_output {
                let file = File::create(output_dir.join("warehouse.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "web_page" => {
            let mut gen = WebPageRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::WebPage);
            if !no_output {
                let file = File::create(output_dir.join("web_page.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        "web_site" => {
            let mut gen = WebSiteRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::WebSite);
            if !no_output {
                let file = File::create(output_dir.join("web_site.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        // Fact tables with child rows (sales + returns)
        // Sales row is first in get_rows(), returns row is second (if present)
        "store_sales" => {
            let mut gen = StoreSalesRowGenerator::new();
            let num_orders = session.get_scaling().get_row_count(Table::StoreSales);
            if !no_output {
                let ss_file = File::create(output_dir.join("store_sales.dat"))?;
                let sr_file = File::create(output_dir.join("store_returns.dat"))?;
                let mut ss_writer = Iso8859Writer::new(BufWriter::new(ss_file));
                let mut sr_writer = Iso8859Writer::new(BufWriter::new(sr_file));
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    // First row is store_sales
                    if !rows.is_empty() {
                        rows[0].write_to(&mut ss_writer, '|')?;
                        row_count += 1;
                    }
                    // Second row (if present) is store_returns
                    if rows.len() > 1 {
                        rows[1].write_to(&mut sr_writer, '|')?;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
                ss_writer.flush()?;
                sr_writer.flush()?;
            } else {
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    if !rows.is_empty() {
                        row_count += 1;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
            }
        }
        "catalog_sales" => {
            let mut gen = CatalogSalesRowGenerator::new();
            let num_orders = session.get_scaling().get_row_count(Table::CatalogSales);
            if !no_output {
                let cs_file = File::create(output_dir.join("catalog_sales.dat"))?;
                let cr_file = File::create(output_dir.join("catalog_returns.dat"))?;
                let mut cs_writer = Iso8859Writer::new(BufWriter::new(cs_file));
                let mut cr_writer = Iso8859Writer::new(BufWriter::new(cr_file));
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    if !rows.is_empty() {
                        rows[0].write_to(&mut cs_writer, '|')?;
                        row_count += 1;
                    }
                    if rows.len() > 1 {
                        rows[1].write_to(&mut cr_writer, '|')?;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
                cs_writer.flush()?;
                cr_writer.flush()?;
            } else {
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    if !rows.is_empty() {
                        row_count += 1;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
            }
        }
        "web_sales" => {
            let mut gen = WebSalesRowGenerator::new();
            let num_orders = session.get_scaling().get_row_count(Table::WebSales);
            if !no_output {
                let ws_file = File::create(output_dir.join("web_sales.dat"))?;
                let wr_file = File::create(output_dir.join("web_returns.dat"))?;
                let mut ws_writer = Iso8859Writer::new(BufWriter::new(ws_file));
                let mut wr_writer = Iso8859Writer::new(BufWriter::new(wr_file));
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    if !rows.is_empty() {
                        rows[0].write_to(&mut ws_writer, '|')?;
                        row_count += 1;
                    }
                    if rows.len() > 1 {
                        rows[1].write_to(&mut wr_writer, '|')?;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
                ws_writer.flush()?;
                wr_writer.flush()?;
            } else {
                let mut row_number = 1i64;
                while row_number <= num_orders {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    let rows = result.get_rows();
                    if !rows.is_empty() {
                        row_count += 1;
                    }
                    if result.should_end_row() {
                        gen.consume_remaining_seeds_for_row();
                        gen.consume_child_seeds();
                        row_number += 1;
                    }
                }
            }
        }
        "inventory" => {
            let mut gen = InventoryRowGenerator::new();
            let num_rows = session.get_scaling().get_row_count(Table::Inventory);
            if !no_output {
                let file = File::create(output_dir.join("inventory.dat"))?;
                let mut writer = Iso8859Writer::new(BufWriter::new(file));
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    for row in result.get_rows() {
                        row.write_to(&mut writer, '|')?;
                        row_count += 1;
                    }
                }
                writer.flush()?;
            } else {
                for row_number in 1..=num_rows {
                    let result =
                        gen.generate_row_and_child_rows(row_number, session, None, None)?;
                    gen.consume_remaining_seeds_for_row();
                    row_count += result.get_rows().len() as i64;
                }
            }
        }
        _ => {
            return Err(format!("Unknown table: {}", table_name).into());
        }
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_millis();
    let rows_per_sec = if duration_ms > 0 {
        row_count as f64 / (duration_ms as f64 / 1000.0)
    } else {
        0.0
    };

    Ok(BenchmarkResult {
        table: table_name.to_string(),
        _scale: session.get_scaling().get_scale(),
        rows: row_count,
        duration_ms,
        rows_per_sec,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let session = create_session(args.scale);

    let tables: Vec<&str> = if args.table == "all" {
        vec![
            "call_center",
            "catalog_page",
            "catalog_sales", // includes catalog_returns
            "customer",
            "customer_address",
            "customer_demographics",
            "date_dim",
            "household_demographics",
            "income_band",
            "inventory",
            "item",
            "promotion",
            "reason",
            "ship_mode",
            "store",
            "store_sales", // includes store_returns
            "time_dim",
            "warehouse",
            "web_page",
            "web_sales", // includes web_returns
            "web_site",
        ]
    } else {
        vec![args.table.as_str()]
    };

    let mut results: Vec<BenchmarkResult> = Vec::new();
    let total_start = Instant::now();

    if !args.json {
        println!("TPC-DS Benchmark - Scale Factor: {}", args.scale);
        println!("============================================");
        println!(
            "{:<25} {:>12} {:>12} {:>15}",
            "Table", "Rows", "Time (ms)", "Rows/sec"
        );
        println!("{:-<65}", "");
    }

    for table in &tables {
        match benchmark_table(table, &session, &args.output_dir, args.no_output) {
            Ok(result) => {
                if !args.json {
                    println!(
                        "{:<25} {:>12} {:>12} {:>15.0}",
                        result.table, result.rows, result.duration_ms, result.rows_per_sec
                    );
                }
                results.push(result);
            }
            Err(e) => {
                eprintln!("Error generating {}: {}", table, e);
            }
        }
    }

    let total_duration = total_start.elapsed();
    let total_rows: i64 = results.iter().map(|r| r.rows).sum();
    let total_ms = total_duration.as_millis();
    let total_rows_per_sec = if total_ms > 0 {
        total_rows as f64 / (total_ms as f64 / 1000.0)
    } else {
        0.0
    };

    if args.json {
        println!("{{");
        println!("  \"scale\": {},", args.scale);
        println!("  \"total_rows\": {},", total_rows);
        println!("  \"total_duration_ms\": {},", total_ms);
        println!("  \"total_rows_per_sec\": {:.0},", total_rows_per_sec);
        println!("  \"tables\": [");
        for (i, result) in results.iter().enumerate() {
            let comma = if i < results.len() - 1 { "," } else { "" };
            println!(
                "    {{\"table\": \"{}\", \"rows\": {}, \"duration_ms\": {}, \"rows_per_sec\": {:.0}}}{}",
                result.table, result.rows, result.duration_ms, result.rows_per_sec, comma
            );
        }
        println!("  ]");
        println!("}}");
    } else {
        println!("{:-<65}", "");
        println!(
            "{:<25} {:>12} {:>12} {:>15.0}",
            "TOTAL", total_rows, total_ms, total_rows_per_sec
        );
        println!("\nTotal time: {:.2}s", total_ms as f64 / 1000.0);
    }

    Ok(())
}
