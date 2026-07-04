use assert_cmd::cargo::cargo_bin_cmd;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::Compression;
use parquet::file::metadata::ParquetMetaDataReader;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use tempfile::tempdir;

/// Test that TPC-DS DAT generation is quiet unless logging is explicitly enabled.
#[test]
fn test_tpcgen_cli_tpcds_dat_is_quiet_by_default() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .env_remove("RUST_LOG")
        .assert()
        .success();

    assert!(
        assert.get_output().stdout.is_empty(),
        "Expected TPC-DS DAT generation to write no stdout by default, got: {}",
        String::from_utf8_lossy(&assert.get_output().stdout)
    );
    assert!(
        assert.get_output().stderr.is_empty(),
        "Expected TPC-DS DAT generation to write no stderr by default, got: {}",
        String::from_utf8_lossy(&assert.get_output().stderr)
    );
}

/// Test that TPC-DS DAT verbose mode enables status logging on stderr.
#[test]
fn test_tpcgen_cli_tpcds_dat_verbose_enables_status_logging() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let assert = cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("-v")
        .env("RUST_LOG", "warn")
        .assert()
        .success();

    assert!(
        assert.get_output().stdout.is_empty(),
        "Expected verbose TPC-DS DAT logging to use stderr, got stdout: {}",
        String::from_utf8_lossy(&assert.get_output().stdout)
    );

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        stderr.contains("Verbose output enabled (ignoring RUST_LOG environment variable)"),
        "Expected verbose mode setup log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("TPC-DS Data Generator (Rust)"),
        "Expected TPC-DS generator status log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("Generating reason..."),
        "Expected TPC-DS table start log, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("Generated reason: 1 rows ->"),
        "Expected TPC-DS table completion log, got stderr: {stderr}"
    );
}

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

/// Test multiple TPC-DS table selection and the default DAT command form.
#[test]
fn test_tpcgen_cli_tpcds_dat_multiple_table_selection_command_forms() {
    let forms: &[&[&str]] = &[&["tpcds"], &["tpcds", "dat"]];

    for form in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen-cli")
            .args(*form)
            .arg("--scale-factor")
            .arg("0")
            .arg("--tables")
            .arg("reason,ship_mode")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success();

        assert!(temp_dir.path().join("reason.dat").exists());
        assert!(temp_dir.path().join("ship_mode.dat").exists());
        assert_eq!(
            fs::read_dir(temp_dir.path())
                .expect("Failed to read generated output directory")
                .count(),
            2,
            "Expected `tpcgen-cli {}` to produce the selected table output set",
            form.join(" ")
        );
    }
}

/// Test that TPC-DS DAT generation forwards compatibility mode to tpcdsgen.
#[test]
fn test_tpcgen_cli_tpcds_dat_compat_mode() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--compat")
        .arg("c")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.dat")).expect("Failed to read DAT file");
    assert_eq!(
        contents.lines().count(),
        75,
        "Expected C compatibility mode to use C dsdgen reason table cardinality"
    );
}

/// Test that TPC-DS DAT generation forwards the actual command line to dbgen_version.
#[test]
fn test_tpcgen_tpcds_dat_dbgen_version_command_line() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("dbgen_version")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents = fs::read_to_string(temp_dir.path().join("dbgen_version.dat"))
        .expect("Failed to read DAT file");
    let fields: Vec<_> = contents
        .trim_end()
        .trim_end_matches('|')
        .split('|')
        .collect();
    assert_eq!(fields.len(), 4);
    assert!(
        fields[3].contains("tpcds dat --scale-factor 1 --tables dbgen_version --output-dir"),
        "Expected dbgen_version command line to contain the actual TPC-DS invocation, got: {}",
        fields[3]
    );
}

/// Test that default DAT output options generate every main TPC-DS output file.
///
/// This overrides only scale factor and output directory: scale factor 0 keeps
/// the integration test fast, while output directory isolates generated files.
#[test]
fn test_tpcgen_cli_tpcds_dat_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("dat")
        .arg("--scale-factor")
        .arg("0")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join("catalog_sales.dat").exists());
    assert!(temp_dir.path().join("catalog_returns.dat").exists());
    assert!(temp_dir.path().join("reason.dat").exists());
    assert_eq!(
        fs::read_dir(temp_dir.path())
            .expect("Failed to read generated output directory")
            .count(),
        25,
        "Expected default TPC-DS DAT generation to produce every main table"
    );
}

/// Test that TPC-DS CSV generation writes a headered CSV file for one table.
#[test]
fn test_tpcgen_cli_tpcds_csv_single_table() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let csv_file = temp_dir.path().join("reason.csv");
    assert!(csv_file.exists(), "Expected {:?} to exist", csv_file);

    let contents = fs::read_to_string(&csv_file).expect("Failed to read CSV file");
    let lines: Vec<_> = contents.lines().collect();
    assert_eq!(
        lines.first(),
        Some(&"r_reason_sk,r_reason_id,r_reason_description")
    );
    assert_eq!(
        lines.len(),
        36,
        "Expected CSV header plus 35 reason rows at scale factor 1"
    );
    assert!(
        lines.iter().all(|line| !line.ends_with(',')),
        "Expected CSV rows not to end with a trailing delimiter, got:\n{contents}"
    );
}

/// Test that TPC-DS CSV generation supports a custom delimiter.
#[test]
fn test_tpcgen_cli_tpcds_csv_custom_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("\\t")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.csv")).expect("Failed to read CSV file");
    let first_line = contents.lines().next().expect("CSV output is empty");
    assert_eq!(first_line, "r_reason_sk\tr_reason_id\tr_reason_description");
    assert!(
        !first_line.contains(','),
        "Expected custom-delimited CSV header not to use commas: {first_line}"
    );
    assert_eq!(
        first_line.matches('\t').count(),
        2,
        "Expected exactly two tab delimiters in the reason header"
    );
}

/// Test that TPC-DS CSV generation escapes headers containing the delimiter.
#[test]
fn test_tpcgen_cli_tpcds_csv_delimiter_in_header_is_escaped() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("_")
        .arg("--scale-factor")
        .arg("1")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let contents =
        fs::read_to_string(temp_dir.path().join("reason.csv")).expect("Failed to read CSV file");
    let first_line = contents.lines().next().expect("CSV output is empty");
    let second_line = contents.lines().nth(1).expect("CSV data row is missing");
    assert_eq!(
        first_line,
        "\"r_reason_sk\"_\"r_reason_id\"_\"r_reason_description\""
    );
    assert_eq!(
        second_line.split('_').count(),
        3,
        "Expected underscore-delimited data rows to have three fields: {second_line}"
    );
}

/// Test that default CSV output options generate every main TPC-DS output file.
#[test]
fn test_tpcgen_cli_tpcds_csv_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    let expected_files: BTreeSet<_> = [
        "call_center.csv",
        "catalog_page.csv",
        "catalog_returns.csv",
        "catalog_sales.csv",
        "customer.csv",
        "customer_address.csv",
        "customer_demographics.csv",
        "date_dim.csv",
        "dbgen_version.csv",
        "household_demographics.csv",
        "income_band.csv",
        "inventory.csv",
        "item.csv",
        "promotion.csv",
        "reason.csv",
        "ship_mode.csv",
        "store.csv",
        "store_returns.csv",
        "store_sales.csv",
        "time_dim.csv",
        "warehouse.csv",
        "web_page.csv",
        "web_returns.csv",
        "web_sales.csv",
        "web_site.csv",
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
        "Expected default TPC-DS CSV generation to produce every main table"
    );
}

/// Test that the TPC-DS CSV subcommand rejects a non-ASCII delimiter at parse time.
#[test]
fn test_tpcgen_cli_tpcds_csv_rejects_non_ascii_delimiter() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen-cli")
        .arg("tpcds")
        .arg("csv")
        .arg("--delimiter")
        .arg("€")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--tables")
        .arg("reason")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("ASCII"));
}
