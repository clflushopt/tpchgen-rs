use assert_cmd::cargo::cargo_bin_cmd;

#[path = "cli_integration/test_helpers.rs"]
mod test_helpers;

// TPCH-specific CLI coverage
#[path = "cli_integration/tpch.rs"]
mod tpch;

// TPC-DS-specific CLI coverage
#[path = "cli_integration/tpcds.rs"]
mod tpcds;

#[cfg(unix)]
#[test]
fn test_tpcgen_cli_rejects_non_utf8_table_names() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    for benchmark in ["tpch", "tpcds"] {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        cargo_bin_cmd!("tpcgen-cli")
            .args([benchmark, "--tables"])
            .arg(OsStr::from_bytes(b"\xff"))
            .arg("--output-dir")
            .arg(temp_dir.path())
            .assert()
            .code(2)
            .stderr(predicates::str::contains(
                "error: table names must be valid UTF-8\n",
            ));
    }
}

/// Test that invoking the CLI without a command reports the top-level usage.
#[test]
fn test_tpcgen_cli_requires_command() {
    cargo_bin_cmd!("tpcgen-cli")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Usage: tpcgen-cli <COMMAND>"))
        .stderr(predicates::str::contains("Commands:"))
        .stderr(predicates::str::contains("tpch"))
        .stderr(predicates::str::contains("tpcds"));
}
