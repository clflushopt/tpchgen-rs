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

/// Test the TPC-DS DAT command forms for the `tpcgen-cli` binary.
#[test]
fn test_tpcgen_cli_tpcds_dat_command_forms() {
    let forms: &[(&[&str], &str)] = &[
        (&["tpcds"], "reason.dat"),
        (&["tpcds", "dat"], "reason.dat"),
    ];

    for (form, expected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output_dir = temp_dir.path().join("generated");

        cargo_bin_cmd!("tpcgen-cli")
            .args(*form)
            .arg("--scale-factor")
            .arg("1")
            .arg("--tables")
            .arg("reason")
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--no-progress")
            .assert()
            .success();

        let expected_file = output_dir.join(expected_file);
        assert!(
            expected_file.exists(),
            "Expected file {:?} to exist with `tpcgen-cli {}`",
            expected_file,
            form.join(" ")
        );

        let contents = fs::read_to_string(&expected_file).expect("Failed to read DAT file");
        assert!(
            contents.starts_with("1|AAAAAAAABAAAAAAA|Package was damaged|\n"),
            "Expected {:?} to contain deterministic pipe-delimited DAT output",
            expected_file
        );
        assert_eq!(
            contents.lines().count(),
            35,
            "Expected {:?} to contain the reason table at scale factor 1",
            expected_file
        );
        assert_eq!(
            fs::read_dir(&output_dir)
                .expect("Failed to read generated output directory")
                .count(),
            1,
            "Expected selected dimension table to produce one DAT file"
        );
    }
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

/// Test that non implemented TPC-DS command forms report generation is unavailable.
#[test]
fn test_tpcgen_cli_tpcds_csv_not_implemented() {
    let forms: &[(&[&str], &[&str], &str)] =
        &[(&["tpcds", "csv"], &["--delimiter", "|"], "reason.csv")];

    for (form, format_args, unexpected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let assert = cargo_bin_cmd!("tpcgen-cli")
            .args(*form)
            .arg("--scale-factor")
            .arg("1")
            .arg("--tables")
            .arg("reason")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .arg("--quiet")
            .args(*format_args)
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(
            stderr.contains("TPC-DS data generation is not yet implemented"),
            "Expected `tpcgen-cli {}` to report that TPC-DS generation is not implemented, got stderr: {}",
            form.join(" "),
            stderr
        );

        let unexpected_file = temp_dir.path().join(unexpected_file);
        assert!(
            !unexpected_file.exists(),
            "Expected `tpcgen-cli {}` not to create {:?}",
            form.join(" "),
            unexpected_file
        );
    }
}