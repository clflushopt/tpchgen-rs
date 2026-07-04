use assert_cmd::cargo::cargo_bin_cmd;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::metadata::ParquetMetaDataReader;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_tpcgen_cli_tpcds_parquet_single_table() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_file = temp_dir.path().join("reason.parquet");
    assert!(expected_file.exists());

    let file = File::open(&expected_file).expect("Failed to open Parquet file");
    let builder =
        ParquetRecordBatchReaderBuilder::try_new(file).expect("Failed to read Parquet metadata");
    assert_eq!(builder.schema().fields().len(), 3);

    let row_count = builder
        .build()
        .expect("Failed to build Parquet reader")
        .map(|batch| batch.expect("Failed to read Parquet batch").num_rows())
        .sum::<usize>();
    assert_eq!(row_count, 35);
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_files: BTreeSet<_> = [
        "call_center.parquet",
        "catalog_page.parquet",
        "catalog_returns.parquet",
        "catalog_sales.parquet",
        "customer.parquet",
        "customer_address.parquet",
        "customer_demographics.parquet",
        "date_dim.parquet",
        "dbgen_version.parquet",
        "household_demographics.parquet",
        "income_band.parquet",
        "inventory.parquet",
        "item.parquet",
        "promotion.parquet",
        "reason.parquet",
        "ship_mode.parquet",
        "store.parquet",
        "store_returns.parquet",
        "store_sales.parquet",
        "time_dim.parquet",
        "warehouse.parquet",
        "web_page.parquet",
        "web_returns.parquet",
        "web_sales.parquet",
        "web_site.parquet",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let actual_files = fs::read_dir(temp_dir.path())
        .expect("Failed to read generated output directory")
        .map(|entry| {
            entry
                .expect("Failed to read generated output directory entry")
                .file_name()
                .into_string()
                .expect("Generated output file name is not valid UTF-8")
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_files, expected_files,
        "Expected default TPC-DS Parquet generation to produce every main table"
    );
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_compression() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--compression")
        .arg("UNCOMPRESSED")
        .assert()
        .success();

    let expected_file = temp_dir.path().join("reason.parquet");
    let file = File::open(&expected_file).expect("Failed to open Parquet file");
    let mut metadata_reader = ParquetMetaDataReader::new();
    metadata_reader.try_parse(&file).unwrap();
    let metadata = metadata_reader.finish().unwrap();

    for row_group in metadata.row_groups() {
        for column in row_group.columns() {
            assert_eq!(column.compression(), Compression::UNCOMPRESSED);
        }
    }
}

#[test]
fn test_tpcgen_cli_tpcds_parquet_unknown_table_error_lists_valid_tables() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("part")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("unknown table 'part'. Expected one of: call_center, catalog_page, catalog_returns, catalog_sales, customer, customer_address, customer_demographics, date_dim, household_demographics, income_band, inventory, item, promotion, reason, ship_mode, store, store_returns, store_sales, time_dim, warehouse, web_page, web_returns, web_sales, web_site, dbgen_version"),
        "Expected unknown table error to list valid TPC-DS tables, got stderr: {stderr}"
    );
}
