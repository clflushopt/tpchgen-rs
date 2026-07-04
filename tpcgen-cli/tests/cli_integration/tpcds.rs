use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

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

/// Test that non-DAT TPC-DS command forms still report that generation is unavailable.
#[test]
fn test_tpcgen_cli_tpcds_non_dat_command_forms_are_not_implemented() {
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
