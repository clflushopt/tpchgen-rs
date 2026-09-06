use assert_cmd::cargo::cargo_bin_cmd;

#[path = "cli_integration/test_helpers.rs"]
mod test_helpers;

// TPCH-specific CLI coverage
#[path = "cli_integration/tpch.rs"]
mod tpch;

// TPC-DS-specific CLI coverage
#[path = "cli_integration/tpcds.rs"]
mod tpcds;

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
