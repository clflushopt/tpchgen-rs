use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::tempdir;

/// Test the TPC-H command forms for the `tpcgen` binary.
#[test]
fn test_tpcgen_tpch_command_forms() {
    let forms: &[(&[&str], &[&str], &str)] = &[
        (&["tpch"], &[], "part.tbl"),
        (&["tpch", "tbl"], &[], "part.tbl"),
        (&["tpch", "csv"], &["--delimiter", "|"], "part.csv"),
        (
            &["tpch", "parquet"],
            &["--compression", "SNAPPY", "--row-group-bytes", "1000000"],
            "part.parquet",
        ),
    ];

    for (form, format_args, expected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen")
            .args(*form)
            .arg("--scale-factor")
            .arg("0.001")
            .arg("--tables")
            .arg("part")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .arg("--no-progress")
            .args(*format_args)
            .assert()
            .success();

        let expected_file = temp_dir.path().join(expected_file);
        assert!(
            expected_file.exists(),
            "Expected file {:?} to exist with `tpcgen {}`",
            expected_file,
            form.join(" ")
        );
    }
}

/// Test the TPC-DS command forms parse and report that generation is unavailable.
#[test]
fn test_tpcgen_tpcds_command_forms_are_not_implemented() {
    let forms: &[(&[&str], &[&str], &str)] = &[
        (&["tpcds"], &[], "reason.dat"),
        (&["tpcds", "dat"], &[], "reason.dat"),
        (&["tpcds", "csv"], &["--delimiter", "|"], "reason.csv"),
        (
            &["tpcds", "parquet"],
            &["--compression", "SNAPPY", "--row-group-bytes", "1000000"],
            "reason.parquet",
        ),
    ];

    for (form, format_args, unexpected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let assert = cargo_bin_cmd!("tpcgen")
            .args(*form)
            .arg("--scale-factor")
            .arg("1")
            .arg("--tables")
            .arg("reason")
            .arg("--output-dir")
            .arg(temp_dir.path())
            .arg("--quiet")
            .arg("--no-progress")
            .args(*format_args)
            .assert()
            .failure();

        let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
        assert!(
            stderr.contains("TPC-DS data generation is not yet implemented"),
            "Expected `tpcgen {}` to report that TPC-DS generation is not implemented, got stderr: {}",
            form.join(" "),
            stderr
        );

        let unexpected_file = temp_dir.path().join(unexpected_file);
        assert!(
            !unexpected_file.exists(),
            "Expected `tpcgen {}` not to create {:?}",
            form.join(" "),
            unexpected_file
        );
    }
}
