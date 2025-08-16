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
    // Create a temporary directory
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

/// Test generating the order table using --parts and --part options
#[test]
fn test_tpchgen_cli_parts() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // generate 4 parts of the orders table with scale factor 0.001
    // into directories /part1, /part2, /part3, /part4
    // use threads to run the command concurrently to minimize the time taken
    let num_parts = 4;
    let mut threads = vec![];
    for part in 1..=num_parts {
        let part_dir = temp_dir.path().join(format!("part{part}"));
        threads.push(std::thread::spawn(move || {
            fs::create_dir(&part_dir).expect("Failed to create part directory");

            // Run the tpchgen-cli command for each part
            Command::cargo_bin("tpchgen-cli")
                .expect("Binary not found")
                .arg("--scale-factor")
                .arg("0.001")
                .arg("--output-dir")
                .arg(&part_dir)
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
    // Read the generated files into a single buffer and compare them
    // to the contents of the reference file
    let mut output_contents = Vec::new();
    for part in 1..=4 {
        let generated_file = temp_dir
            .path()
            .join(format!("part{part}"))
            .join("orders.tbl");
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
    let reference_file = read_reference_file("orders", "0.001");
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
                row_group_bytes: vec![6524595, 6510817, 6509412, 6519647],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    7159834, 7109252, 7093240, 7123300, 7147731, 7122707, 7144719, 7101681,
                    7113659, 7109747, 7109526, 7143030, 7105585, 7100415, 7143142, 7117154,
                    7147556, 7115410, 7109609, 7096825, 7111561, 7155528, 7108907, 7110276,
                    7147443, 7103508, 7113014, 7129395, 7120851, 7160720, 7125178, 7137503,
                    7117439, 7116240, 7120922, 7098800, 7132250, 7126634, 7118900, 7108375,
                    7126762, 7145664, 7104909, 7132885, 7103637, 7103739, 7142231, 7111048,
                    7093823, 7096310, 7160884, 7159874, 7135249,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2931],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![
                    7843809, 7843770, 7849113, 7846008, 7850945, 7848848, 7840156, 7842590,
                    7841844, 7840741, 7842821, 7841010, 7845089, 7835475, 7841544, 7839733,
                ],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![7015205, 7016059],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![
                    7296158, 7278688, 7293668, 7289456, 7287098, 7294237, 7281630, 7302815,
                    7286591, 7292998, 7288736, 7299556, 7295055, 7297254, 7292243, 7281443,
                ],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![756],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1637351],
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
                row_group_bytes: vec![12849948, 12843398],
            },
            RowGroups {
                table: "lineitem",
                row_group_bytes: vec![
                    18120705, 18173631, 18121037, 18098532, 18104251, 18159584, 18143034, 18087872,
                    18116882, 18146572, 18137133, 18192817, 18109983, 18107897, 18137448, 18126493,
                    18125071, 18120274, 18113389, 18177883,
                ],
            },
            RowGroups {
                table: "nation",
                row_group_bytes: vec![2931],
            },
            RowGroups {
                table: "orders",
                row_group_bytes: vec![19819201, 19823559, 19814276, 19810582, 19806184, 19799240],
            },
            RowGroups {
                table: "part",
                row_group_bytes: vec![13923638],
            },
            RowGroups {
                table: "partsupp",
                row_group_bytes: vec![18983980, 18996831, 18979486, 18982575, 19001482, 18988488],
            },
            RowGroups {
                table: "region",
                row_group_bytes: vec![756],
            },
            RowGroups {
                table: "supplier",
                row_group_bytes: vec![1637351],
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
fn test_tpchgen_cli_parts_no_part() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // CLI Error test --parts and but not --part
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--output-dir")
        .arg(temp_dir.path())
        .arg("--parts")
        .arg("42")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "The --part_count option requires the --part option to be set",
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
