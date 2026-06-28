use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
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

/// Test the TPC-DS DAT command forms for the `tpcgen` binary.
#[test]
fn test_tpcgen_tpcds_dat_command_forms() {
    let forms: &[(&[&str], &str)] = &[
        (&["tpcds"], "reason.dat"),
        (&["tpcds", "dat"], "reason.dat"),
    ];

    for (form, expected_file) in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let output_dir = temp_dir.path().join("generated");

        cargo_bin_cmd!("tpcgen")
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
            "Expected file {:?} to exist with `tpcgen {}`",
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

/// Test multiple TPC-DS table selection and the default DAT command form.
#[test]
fn test_tpcgen_tpcds_dat_multiple_table_selection_command_forms() {
    let forms: &[&[&str]] = &[&["tpcds"], &["tpcds", "dat"]];

    for form in forms {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        cargo_bin_cmd!("tpcgen")
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
            "Expected `tpcgen {}` to produce the selected table output set",
            form.join(" ")
        );
    }
}

/// Test that default DAT output options generate every main TPC-DS output file.
///
/// This overrides only scale factor and output directory: scale factor 0 keeps
/// the integration test fast, while output directory isolates generated files.
#[test]
fn test_tpcgen_tpcds_dat_default_options_generate_all_outputs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    cargo_bin_cmd!("tpcgen")
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

/// Test that non-DAT TPC-DS command forms still report that generation is unavailable.
#[test]
fn test_tpcgen_tpcds_non_dat_command_forms_are_not_implemented() {
    let forms: &[(&[&str], &[&str], &str)] = &[
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
