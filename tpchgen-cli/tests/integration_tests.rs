use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Read;
use tempfile::tempdir;

/// Test TBL output for scale factor 0.01 using tpchgen-cli
#[test]
fn test_tpchgen_cli_scale_factor_0_01() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    // Run the tpchgen-cli command
    Command::cargo_bin("tpchgen-cli")
        .expect("Binary not found")
        .arg("--scale-factor")
        .arg("0.01")
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
        let reference_file = format!("../tpchgen/data/sf-0.01/{}.gz", file);
        let reference_contents = match read_gzipped_file(&reference_file) {
            Ok(contents) => contents,
            Err(e) => {
                panic!("Failed to read reference file {reference_file}: {e}");
            }
        };
        let reference_contents = String::from_utf8(reference_contents)
            .expect("Failed to convert reference contents to string");

        assert_eq!(
            generated_contents, reference_contents,
            "Contents of {:?} do not match reference",
            file
        );
    }
}

fn read_gzipped_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, std::io::Error> {
    let file = fs::File::open(path)?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents)?;
    Ok(contents)
}
