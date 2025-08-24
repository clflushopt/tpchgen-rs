use assert_cmd::Command;
use parquet::arrow::arrow_reader::{ArrowReaderOptions, ParquetRecordBatchReaderBuilder};
use parquet::file::metadata::ParquetMetaDataReader;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use tpchgen::generators::OrderGenerator;
use tpchgen_arrow::{OrderArrow, RecordBatchIterator};

/// Test TBL output for scale factor 0.001 using tpchgen-cli
#[test]
fn test_tpchgen_cli_tbl_scale_factor_0_001() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // Run the tpchgen-cli command
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .assert()
        .success();

    // List of expected files
    let expected_files = vec![
        "customer.tbl",
        "lineitem.tbl",
        "nation.tbl",
        "orders.tbl",
        "part.tbl",
        "partsupp.tbl",
        "region.tbl",
        "supplier.tbl",
    ];

    // Verify that all expected files are created
    for file in &expected_files {
        let generated_file = temp_dir.path().join(file);
        assert!(
            generated_file.exists(),
            "File {:?} does not exist",
            generated_file
        );
        let generated_contents = fs::read(generated_file).expect("Failed to read generated file");
        let generated_contents = String::from_utf8(generated_contents)
            .expect("Failed to convert generated contents to string");

        // load the reference file
        let reference_file = format!("../tpchgen/data/sf-0.001/{}.gz", file);
        let reference_contents = match read_gzipped_file_to_string(&reference_file) {
            Ok(contents) => contents,
            Err(e) => {
                panic!("Failed to read reference file {reference_file}: {e}");
            }
        };

        assert_eq!(
            generated_contents, reference_contents,
            "Contents of {:?} do not match reference",
            file
        );
    }
}

/// Test that when creating output, if the file already exists it is not overwritten
#[test]
fn test_tpchgen_cli_tbl_no_overwrite() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let expected_file = temp_dir.path().join("part.tbl");

    let run_command = || {
        Command::cargo_bin("tpchgen-cli")
            .expect("Binary not found")
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg("part")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success()
    };

    run_command();
    let original_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), 23498);

    // Run the tpchgen-cli command again with the same parameters and expect the
    // file to not be overwritten
    run_command();
    let new_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), new_metadata.len());
    assert_eq!(
        original_metadata
            .modified()
            .expect("Failed to get modified time"),
        new_metadata
            .modified()
            .expect("Failed to get modified time")
    );
}

// Test that when creating output, if the file already exists it is not for parquet
#[test]
fn test_tpchgen_cli_parquet_no_overwrite() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let expected_file = temp_dir.path().join("part.parquet");

    let run_command = || {
        Command::cargo_bin("tpchgen-cli")
            .expect("Binary not found")
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg("part")
            .arg("--format")
            .arg("parquet")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .success()
    };

    run_command();
    let original_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), 12061);

    // Run the tpchgen-cli command again with the same parameters and expect the
    // file to not be overwritten
    run_command();

    let new_metadata =
        fs::metadata(&expected_file).expect("Failed to get metadata of generated file");
    assert_eq!(original_metadata.len(), new_metadata.len());
    assert_eq!(
        original_metadata
            .modified()
            .expect("Failed to get modified time"),
        new_metadata
            .modified()
            .expect("Failed to get modified time")
    );
}

/// Test generating the order table using 4 parts implicitly
#[test]
fn test_tpchgen_cli_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // generate 4 parts of the orders table with scale factor 0.001 and let
    // tpchgen-cli generate the multiple files

    let num_parts = 4;
    let output_dir = temp_dir.path().to_path_buf();
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--parts")
        .arg(num_parts.to_string())
        .arg("--tables")
        .arg("orders")
        .assert()
        .success();

    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
}

/// Test generating the order table with multiple invocations using --parts and
/// --part options
#[test]
fn test_tpchgen_cli_parts_explicit() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // generate 4 parts of the orders table with scale factor 0.001
    // use threads to run the command concurrently to minimize the time taken
    let num_parts = 4;
    let mut threads = vec![];
    for part in 1..=num_parts {
        let output_dir = temp_dir.path().to_path_buf();
        threads.push(std::thread::spawn(move || {
            // Run the tpchgen-cli command for each part
            // output goes into `output_dir/orders/orders.{part}.tbl`
            Command::cargo_bin("tpchgen-cli")
                .expect("Binary not found")
                .arg("--scale-factor")
                .arg("0.001")
                .arg("--output-dir")
                .arg(&output_dir)
                .arg("--parts")
                .arg(num_parts.to_string())
                .arg("--part")
                .arg(part.to_string())
                .arg("--tables")
                .arg("orders")
                .assert()
                .success();
        }));
    }
    // Wait for all threads to finish
    for thread in threads {
        thread.join().expect("Thread panicked");
    }
    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
}

/// Create all tables using --parts option and verify the output layouts
#[test]
fn test_tpchgen_cli_parts_all_tables() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let num_parts = 8;
    let output_dir = temp_dir.path().to_path_buf();
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--parts")
        .arg(num_parts.to_string())
        .assert()
        .success();

    verify_table(temp_dir.path(), "lineitem", num_parts, "0.001");
    verify_table(temp_dir.path(), "orders", num_parts, "0.001");
    verify_table(temp_dir.path(), "part", num_parts, "0.001");
    verify_table(temp_dir.path(), "partsupp", num_parts, "0.001");
    verify_table(temp_dir.path(), "customer", num_parts, "0.001");
    verify_table(temp_dir.path(), "supplier", num_parts, "0.001");
    // Note, nation and region have only a single part regardless of --parts
    verify_table(temp_dir.path(), "nation", 1, "0.001");
    verify_table(temp_dir.path(), "region", 1, "0.001");
}

/// Read the N files from `output_dir/table_name/table_name.part.tml` into a
/// single buffer and compare them to the contents of the reference file
fn verify_table(output_dir: &Path, table_name: &str, parts: usize, scale_factor: &str) {
    let mut output_contents = Vec::new();
    for part in 1..=parts {
        let generated_file = output_dir
            .join(table_name)
            .join(format!("{table_name}.{part}.tbl"));
        assert!(
            generated_file.exists(),
            "File {:?} does not exist",
            generated_file
        );
        let generated_contents =
            fs::read_to_string(generated_file).expect("Failed to read generated file");
        output_contents.append(&mut generated_contents.into_bytes());
    }
    let output_contents =
        String::from_utf8(output_contents).expect("Failed to convert output contents to string");

    // load the reference file
    let reference_file = read_reference_file(table_name, scale_factor);
    assert_eq!(output_contents, reference_file);
}

#[tokio::test]
async fn test_write_parquet_orders() {
    // Run the CLI command to generate parquet data
    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("orders.parquet");
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--format")
        .arg("parquet")
        .arg("--tables")
        .arg("orders")
        .arg("--scale-factor")
        .arg("0.001")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    let batch_size = 4000;

    // Create the reference Arrow data using OrderArrow
    let generator = OrderGenerator::new(0.001, 1, 1);
    let mut arrow_generator = OrderArrow::new(generator).with_batch_size(batch_size);

    // Read the generated parquet file
    let file = File::open(&output_path).expect("Failed to open parquet file");
    let options = ArrowReaderOptions::new().with_schema(Arc::clone(arrow_generator.schema()));

    let reader = ParquetRecordBatchReaderBuilder::try_new_with_options(file, options)
        .expect("Failed to create ParquetRecordBatchReaderBuilder")
        .with_batch_size(batch_size)
        .build()
        .expect("Failed to build ParquetRecordBatchReader");

    // Compare the record batches
    for batch in reader {
        let parquet_batch = batch.expect("Failed to read record batch from parquet");
        let arrow_batch = arrow_generator
            .next()
            .expect("Failed to generate record batch from OrderArrow");
        assert_eq!(
            parquet_batch, arrow_batch,
            "Mismatch between parquet and arrow record batches"
        );
    }
}

#[tokio::test]
async fn test_write_parquet_row_group_size_default() {
    // Run the CLI command to generate parquet data with default settings
    let output_dir = tempdir().unwrap();
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--format")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    expect_row_group_sizes(
        output_dir.path(),
        vec![
            RowGroups {
                table: "customer",
                row_group_bytes: vec![6523694, 6509728, 6508417, 6518582],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    7157554, 7106900, 7090842, 7120906, 7145325, 7120319, 7142364, 7099258,
                    7111326, 7107355, 7107174, 7140691, 7103258, 7098064, 7140780, 7114738,
                    7145231, 7112989, 7107260, 7094419, 7109164, 7153132, 7106588, 7107901,
                    7145001, 7101142, 7110720, 7127039, 7118498, 7158328, 7122729, 7135124,
                    7115110, 7113817, 7118599, 7096420, 7129813, 7124217, 7116502, 7105980,
                    7124396, 7143315, 7102503, 7130464, 7101232, 7101367, 7139904, 7108710,
                    7091458, 7093976, 7158507, 7157452, 7132894,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2684],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![
                    7842293, 7842276, 7847651, 7844507, 7849468, 7847495, 7838699, 7841044,
                    7840487, 7839166, 7841356, 7839460, 7843712, 7834117, 7840051, 7838256,
                ],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![7013437, 7014324],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![
                    7294343, 7277020, 7291816, 7287548, 7285532, 7292484, 7279927, 7300829,
                    7284682, 7291052, 7286819, 7297626, 7292903, 7295373, 7290239, 7279755,
                ],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![554],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1636998],
            },
        ],
    );
}

#[tokio::test]
async fn test_write_parquet_row_group_size_20mb() {
    // Run the CLI command to generate parquet data with larger row group size
    let output_dir = tempdir().unwrap();
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--format")
        .arg("parquet")
        .arg("--scale-factor")
        .arg("1")
        .arg("--output-dir")
        .arg(output_dir.path())
        .arg("--parquet-row-group-bytes")
        .arg("20000000") // 20 MB
        .assert()
        .success();

    expect_row_group_sizes(
        output_dir.path(),
        vec![
            RowGroups {
                table: "customer",
                row_group_bytes: vec![12848012, 12841373],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    18114785, 18167648, 18114968, 18092636, 18098372, 18153536, 18137038, 18081920,
                    18110927, 18140643, 18131304, 18186767, 18103994, 18101890, 18131440, 18120528,
                    18119019, 18114395, 18107484, 18171954,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2684],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![19815261, 19819775, 19810468, 19806802, 19802354, 19795478],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![13920228],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![18979513, 18992386, 18975044, 18978110, 18996633, 18983782],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![554],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1636998],
            },
        ],
    );
}

#[test]
fn test_tpchgen_cli_part_no_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // CLI Error test --part and but not --parts
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("42")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "The --part option requires the --parts option to be set",
        ));
}

#[test]
fn test_tpchgen_cli_too_many_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // This should fail because --part is 42 which is more than the --parts 10
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("42")
        .arg("--parts")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected at most the value of --parts (10), got 42",
        ));
}

#[test]
fn test_tpchgen_cli_zero_part() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("0")
        .arg("--parts")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected a number greater than zero, got 0",
        ));
}
#[test]
fn test_tpchgen_cli_zero_part_zero_parts() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--part")
        .arg("0")
        .arg("--parts")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Invalid --part. Expected a number greater than zero, got 0",
        ));
}

/// Test specifying parquet options even when writing tbl output
#[tokio::test]
async fn test_incompatible_options_warnings() {
    let output_dir = tempdir().unwrap();
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--format")
        .arg("csv")
        .arg("--tables")
        .arg("orders")
        .arg("--scale-factor")
        .arg("0.0001")
        .arg("--output-dir")
        .arg(output_dir.path())
        // pass in parquet options that are incompatible with csv
        .arg("--parquet-compression")
        .arg("zstd(1)")
        .arg("--parquet-row-group-bytes")
        .arg("8192")
        .assert()
        // still success, but should see warnints
        .success()
        .stderr(predicates::str::contains(
            "Warning: Parquet compression option set but not generating Parquet files",
        ))
        .stderr(predicates::str::contains(
            "Warning: Parquet row group size option set but not generating Parquet files",
        ));
}

fn read_gzipped_file_to_string<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;
    let contents = String::from_utf8(contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(contents)
}

/// Reads the reference file for the specified table and scale factor.
///
/// example usage: `read_reference_file("orders", "0.001")`
fn read_reference_file(table_name: &str, scale_factor: &str) -> String {
    let reference_file = format!("../tpchgen/data/sf-{scale_factor}/{table_name}.tbl.gz");
    match read_gzipped_file_to_string(&reference_file) {
        Ok(contents) => contents,
        Err(e) => {
            panic!("Failed to read reference file {reference_file}: {e}");
        }
    }
}

#[derive(Debug, PartialEq)]
struct RowGroups {
    table: &'static str,
    /// total bytes in each row group
    row_group_bytes: Vec<i64>,
}

/// For each table in tables, check that the parquet file in output_dir has
/// a file with the expected row group sizes.
fn expect_row_group_sizes(output_dir: &Path, expected_row_groups: Vec<RowGroups>) {
    let mut actual_row_groups = vec![];
    for table in &expected_row_groups {
        let output_path = output_dir.join(format!("{}.parquet", table.table));
        assert!(
            output_path.exists(),
            "Expected parquet file {:?} to exist",
            output_path
        );
        // read the metadata to get the row group size
        let file = File::open(&output_path).expect("Failed to open parquet file");
        let mut metadata_reader = ParquetMetaDataReader::new();
        metadata_reader.try_parse(&file).unwrap();
        let metadata = metadata_reader.finish().unwrap();
        let row_groups = metadata.row_groups();
        let actual_row_group_bytes: Vec<_> =
            row_groups.iter().map(|rg| rg.total_byte_size()).collect();
        actual_row_groups.push(RowGroups {
            table: table.table,
            row_group_bytes: actual_row_group_bytes,
        })
    }
    // compare the expected and actual row groups debug print actual on failure
    // for better output / easier comparison
    let expected_row_groups = format!("{expected_row_groups:#?}");
    let actual_row_groups = format!("{actual_row_groups:#?}");
    assert_eq!(actual_row_groups, expected_row_groups);
}
