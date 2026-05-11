# TPC-DS Test Scripts

This directory contains scripts for testing the Rust TPC-DS implementation
against two reference implementations:

1. **Java / Trino** (default, `--compat trino`) — the Java port of `dsdgen`
   used by Trino. The Rust port was originally derived from this and is
   expected to be byte-for-byte identical.
2. **C `dsdgen`** (`--compat c`) — the original TPC-supplied reference
   implementation. The `--compat c` mode corrects bugs in the Java port to
   match the C reference (see [BUGS.md](../BUGS.md) and the parent
   [README](../README.md)).

Both conformance suites validate **byte-for-byte identical** output via
MD5/`diff` comparison.

## Directory Structure

```
tpcdsgen/
├── tests/
│   └── fixtures/                # Reference data (gitignored)
│       ├── scale-1-java/        # Java reference (`--compat trino`)
│       │   ├── call_center.dat
│       │   ├── warehouse.dat
│       │   └── ... (all 25 tables)
│       └── scale-1-c/           # C dsdgen reference (`--compat c`)
│           ├── call_center.dat
│           ├── warehouse.dat
│           └── ... (all 25 tables)
└── scripts/
    ├── bootstrap-java.sh        # Clone + build the Java TPC-DS impl
    ├── bootstrap-c.sh           # Download C dsdgen pre-generated data
    ├── generate-fixtures.sh     # Generate Java reference fixtures
    ├── compare-table.sh         # Compare one table
    ├── test-all-tables.sh       # Compare all ported tables
    ├── clean-fixtures.sh        # Clean fixtures
    └── README.md                # This file
```

## Quick Start — Java conformance (`--compat trino`)

```bash
# 1. Bootstrap Java implementation (first time only)
./scripts/bootstrap-java.sh

# 2. Generate Java reference fixtures into tests/fixtures/scale-N-java/.
./scripts/generate-fixtures.sh

# 3. Test all ported tables against the Java reference.
./scripts/test-all-tables.sh
```

## Quick Start — C dsdgen conformance (`--compat c`)

The C reference data is pre-generated and published in
[alamb/tpcds-data](https://github.com/alamb/tpcds-data), one branch per
scale factor (`sf1`, `sf2`, ...). The bootstrap script clones the requested
branch with `--depth 1` and extracts it into `tests/fixtures/scale-N-c/`.

```bash
# 1. Download the C dsdgen reference data (default scale 1).
./scripts/bootstrap-c.sh                # sf1
./scripts/bootstrap-c.sh --scale 2      # sf2

# 2. Test all ported tables against the C reference.
./scripts/test-all-tables.sh --compat c

# Or compare a single table.
./scripts/compare-table.sh reason --compat c
```

### Tables excluded from automated checks

The following tables are excluded from automated MD5 comparison; the
exclusion lists live in `test-all-tables.sh`.

- **Always:** `dbgen_version.dat` — contains a generation timestamp.
- **`--compat c` only:** `customer.dat` — the reference data in
  `alamb/tpcds-data` was generated through a pipeline that double-UTF-8
  encodes the non-ASCII country names (`CÔTE D'IVOIRE`, `RÉUNION`). The
  Rust `--compat c` output uses raw Latin-1, which is what unmodified C
  `dsdgen` produces. Once the reference data is regenerated without the
  `iconv ISO-8859-14 -> UTF-8` step in `alamb/tpcds-data`'s `Dockerfile`,
  this exclusion can be removed.

## Scripts

### 0. `bootstrap-java.sh` - Setup Java TPC-DS Implementation

**⚠️ Run this first!** Sets up the Java TPC-DS implementation needed for conformance testing.

**Usage:**
```bash
# First time setup (clone and build)
./scripts/bootstrap-java.sh

# Force rebuild
./scripts/bootstrap-java.sh --rebuild

# Verify existing installation
./scripts/bootstrap-java.sh --verify

# Show help
./scripts/bootstrap-java.sh --help
```

**What it does:**
1. Checks if Java and Maven are installed
2. Clones the Java TPC-DS repository from GitHub (if needed)
3. Builds the Java implementation with Maven
4. Runs a smoke test to verify it works

**Requirements:**
- Java 11+ (e.g., `brew install openjdk@11`)
- Maven (e.g., `brew install maven`)
- Git

**Environment Variables:**
- `TPCDS_JAVA_REPO` - Override the Java repo URL (default: https://github.com/trinodb/tpcds.git)

**Output:**
- Clones to `../tpcds/` (parallel to this repo)
- Creates `../tpcds/target/tpcds-*-jar-with-dependencies.jar`

**Time:** ~2-3 minutes (first run)

### 0b. `bootstrap-c.sh` - Download C dsdgen Reference Data

Downloads pre-generated TPC-DS data from
[alamb/tpcds-data](https://github.com/alamb/tpcds-data) and extracts it
into `tests/fixtures/scale-N-c/`. Each scale factor lives on its own branch
(`sf1`, `sf2`, ...), packaged as split bzip2 tarballs. Only the requested
branch is cloned (`git clone --depth 1 --single-branch`).

**Usage:**
```bash
./scripts/bootstrap-c.sh                # Download sf1
./scripts/bootstrap-c.sh --scale 2      # Download sf2
./scripts/bootstrap-c.sh --rebuild      # Force re-download
./scripts/bootstrap-c.sh --verify       # Verify existing fixtures
./scripts/bootstrap-c.sh --help
```

**Requirements:** `git`, `tar`, `bzip2`.

**Environment Variables:**
- `TPCDS_C_DATA_REPO` — override the data repo URL
  (default: `https://github.com/alamb/tpcds-data.git`).

**Output:** `tests/fixtures/scale-N-c/*.dat` (25 tables).

**Time:** ~30 seconds at SF1 (~315 MB compressed, ~2.4 GB extracted).

---

### 1. `generate-fixtures.sh` - Generate Reference Data

Generates TPC-DS tables using the Java implementation. This creates the "golden reference" data that Rust output is compared against.

**Usage:**
```bash
# Generate all 25 tables (recommended first run)
./scripts/generate-fixtures.sh

# Generate specific tables
./scripts/generate-fixtures.sh call_center warehouse

# Quiet mode (minimal output)
./scripts/generate-fixtures.sh --quiet

# Show help
./scripts/generate-fixtures.sh --help
```

**What it does:**
1. Checks if Java implementation is built (builds if needed)
2. Creates `tests/fixtures/scale-N-java/` directory
3. Generates each table using Java TPC-DS generator
4. Reports progress and statistics

**Output:**
- Generates `.dat` files in `tests/fixtures/scale-N-java/`
- Each file contains pipe-delimited rows with trailing pipe: `value1|value2|value3|`
- Files are gitignored (regenerate as needed)

**Time:** ~2-5 minutes for all 25 tables at scale 1

---

### 2. `compare-table.sh` - Compare Single Table

Compares Rust-generated output for a single table against either the Java
or the C dsdgen reference fixture.

**Usage:**
```bash
# Compare against Java fixture (default).
./scripts/compare-table.sh call_center

# Compare against C dsdgen fixture.
./scripts/compare-table.sh reason --compat c

# Quiet mode
./scripts/compare-table.sh customer_demographics --quiet

# Show help
./scripts/compare-table.sh --help
```

**What it does:**
1. Looks up the appropriate fixture directory based on `--compat`:
   - `--compat trino` (default): `tests/fixtures/scale-N-java/`
   - `--compat c`: `tests/fixtures/scale-N-c/`
2. Generates the table using the Rust implementation in matching compat mode.
3. Performs MD5 + `diff` comparison.
4. Reports results.

**Exit codes:**
- `0` - Tables match exactly ✓
- `1` - Tables differ or error occurred ✗

**Output example:**
```
[INFO] =========================================
[INFO] Table Comparison: call_center
[INFO] =========================================
[INFO] Java fixture: tests/fixtures/scale-1-java/call_center.dat
[INFO] Generating call_center with Rust...
[INFO] Using binary: target/release/tpcdsgen --table call_center
[INFO] Comparing outputs...
[INFO] Java fixture: 6 rows, 4.0K
[INFO] Rust output:  6 rows, 4.0K
[SUCCESS] ✓ call_center: MD5 match (6 rows, cc9aabc63eb8603bd7330b6735ed0961)
[INFO] =========================================
```

---

### 3. `test-all-tables.sh` - Test All Ported Tables

Runs comparison tests for all tables that have been ported to Rust. This is
the main test suite for both conformance modes.

**Usage:**
```bash
# Test all tables against Java reference (default).
./scripts/test-all-tables.sh

# Test all tables against C dsdgen reference.
./scripts/test-all-tables.sh --compat c

# Quiet mode (show only summary).
./scripts/test-all-tables.sh --quiet

# Show help
./scripts/test-all-tables.sh --help
```

**What it does:**
1. Tests all 24 TPC-DS tables (`dbgen_version` always excluded — timestamps).
   For `--compat c`, `customer` is also excluded (see top of this README).
2. Builds the unified Rust generator (`tpcdsgen`).
3. Compares each table against the configured reference using `compare-table.sh`.
4. Prints comprehensive summary.

**Exit codes:**
- `0` - All tables match ✓
- `1` - One or more tables differ ✗

**Output example:**
```
[INFO] =========================================
[INFO] TPC-DS Table Test Suite
[INFO] =========================================
[INFO] Testing 24 tables:
[INFO]   - call_center
[INFO]   - catalog_page
[INFO]   - catalog_returns
[INFO]   ... (all 24 tables)
[INFO] =========================================
[INFO] Building Rust TPC-DS generator...
[SUCCESS] Generator built successfully
[INFO] =========================================

[INFO] Testing: call_center
...
[SUCCESS] ✓ call_center: MD5 match (6 rows, cc9aabc63eb8603bd7330b6735ed0961)
...

[INFO] =========================================
[INFO] Test Summary
[INFO] =========================================
[INFO] Total tables tested: 24
[SUCCESS] Passed: 24

[INFO] Total time: 45s
[INFO] =========================================
```

---

### 4. `clean-fixtures.sh` - Clean Up Fixtures

Removes all generated fixtures to free up disk space or force regeneration.

**Usage:**
```bash
# Clean with confirmation prompt
./scripts/clean-fixtures.sh

# Clean without confirmation
./scripts/clean-fixtures.sh --yes

# Show help
./scripts/clean-fixtures.sh --help
```

**What it does:**
1. Counts fixture files and reports total size
2. Asks for confirmation (unless `--yes` provided)
3. Deletes entire `tests/fixtures/` directory

---

## Typical Workflow

### Java conformance
```bash
# 1. Generate Java reference fixtures (one-time, or when Java changes).
./scripts/generate-fixtures.sh

# 2. Run the comparison.
./scripts/compare-table.sh <table>     # one table
./scripts/test-all-tables.sh           # all tables
```

### C dsdgen conformance
```bash
# 1. Download the C reference data (one-time, or to refresh).
./scripts/bootstrap-c.sh

# 2. Run the comparison in C-compat mode.
./scripts/compare-table.sh <table> --compat c
./scripts/test-all-tables.sh --compat c
```

### Cleanup
```bash
./scripts/clean-fixtures.sh --yes      # remove all fixtures
```

---

## Requirements

- **Java:** Maven-built TPC-DS JAR at `../tpcds/target/tpcds-*-jar-with-dependencies.jar` (`bootstrap-java.sh` handles this).
- **C dsdgen reference:** `git`, `tar`, `bzip2` for `bootstrap-c.sh`. No compiler required — data is pre-generated.
- **Rust:** Cargo-built `tpcdsgen` binary at `target/debug/tpcdsgen` or `target/release/tpcdsgen`.
- **Disk space:** ~1 GB for SF1 Java fixtures; ~2.4 GB for SF1 C fixtures.

---

## Troubleshooting

**Problem:** `Java JAR not found`
```bash
cd ../tpcds
mvn clean package
```

**Problem:** `Rust binary not found`
```bash
cargo build --release
```

**Problem:** `Fixture not found` (Java path)
```bash
./scripts/generate-fixtures.sh X
```

**Problem:** `Fixture not found` (C path)
```bash
./scripts/bootstrap-c.sh --scale N
```

**Problem:** Tables don't match
1. Check that the right compat mode is selected (`--compat trino` vs `--compat c`).
2. Verify both sides use the same seed (the Rust generator is deterministic).
3. Use the `diff` output to find the first difference.
4. Debug the specific row/column that differs.

---

## Integration with CI/CD

These scripts are designed to be CI-friendly:

```yaml
# Java conformance
- run: ./scripts/bootstrap-java.sh
- run: ./scripts/generate-fixtures.sh --quiet
- run: ./scripts/test-all-tables.sh --quiet

# C dsdgen conformance
- run: ./scripts/bootstrap-c.sh
- run: ./scripts/test-all-tables.sh --compat c --quiet
```

Exit codes make it easy to fail CI on mismatches.

## Notes

- **Fixtures are gitignored** - They're generated artifacts, not source code
- **Deterministic output** - Same seed always produces same data
- **Byte-for-byte equality** - Not just row count, complete binary match
- **Bug compatibility** - Maintains same quirks as Java/C versions (e.g., leap year bug)
