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
│       ├── scale-1-trino/        # Java reference (`--compat trino`)
│       │   ├── call_center.dat
│       │   ├── warehouse.dat
│       │   └── ... (all 25 tables)
│       └── scale-1-c/           # C dsdgen reference (`--compat c`)
│           ├── call_center.dat
│           ├── warehouse.dat
│           └── ... (all 25 tables)
└── scripts/
    ├── bootstrap-trino.sh        # Clone + build the Java TPC-DS impl
    ├── generate-fixtures.sh     # Generate/download reference fixtures
    │                            #   (Java via --compat trino; C via --compat c)
    ├── compare-table.sh         # Compare one table
    ├── compare-all-tables.sh       # Compare all ported tables
    ├── clean-fixtures.sh        # Clean fixtures
    └── README.md                # This file
```

## Quick Start — Java conformance (`--compat trino`)

```bash
# 1. Bootstrap Java implementation (first time only)
./scripts/bootstrap-trino.sh

# 2. Generate Java reference fixtures into tests/fixtures/scale-N-trino/.
./scripts/generate-fixtures.sh

# 3. Test all ported tables against the Java reference.
./scripts/compare-all-tables.sh
```

## Quick Start — C dsdgen conformance (`--compat c`)

The C reference data is pre-generated and published in
[alamb/tpcds-data](https://github.com/alamb/tpcds-data), one branch per
scale factor (`sf1`, `sf2`, ...). `generate-fixtures.sh --compat c` clones
the requested branch with `--depth 1` and extracts it into
`tests/fixtures/scale-N-c/`.

```bash
# 1. Download the C dsdgen reference data (default scale 1).
./scripts/generate-fixtures.sh --compat c              # sf1
./scripts/generate-fixtures.sh --compat c --scale 2    # sf2

# 2. Test all ported tables against the C reference.
./scripts/compare-all-tables.sh --compat c

# Or compare a single table.
./scripts/compare-table.sh reason --compat c
```

### Tables excluded from automated checks

The following tables are excluded from automated MD5 comparison; the
exclusion lists live in `compare-all-tables.sh`.

- **Always:** `dbgen_version.dat` — contains a generation timestamp.
- **`--compat c` only:** `customer.dat` — the reference data in
  `alamb/tpcds-data` was generated through a pipeline that double-UTF-8
  encodes the non-ASCII country names (`CÔTE D'IVOIRE`, `RÉUNION`). The
  Rust `--compat c` output uses raw Latin-1, which is what unmodified C
  `dsdgen` produces. Once the reference data is regenerated without the
  `iconv ISO-8859-14 -> UTF-8` step in `alamb/tpcds-data`'s `Dockerfile`,
  this exclusion can be removed.

## Scripts

Each script is self-documenting — open it and read the header comment for
full usage, flags, environment variables, output, and exit codes. The
table below is just a roadmap.

| Script                    | Purpose                                                                                                                         |
|---------------------------|---------------------------------------------------------------------------------------------------------------------------------|
| `bootstrap-trino.sh`       | Clone and build the Java / Trino reference implementation into `../tpcds/`. Run once before Java conformance.                   |
| `generate-fixtures.sh`    | Populate `tests/fixtures/scale-N-{trino,c}/` with reference data. `--compat trino` (default) runs the Java impl; `--compat c` downloads pre-generated C `dsdgen` data from [alamb/tpcds-data](https://github.com/alamb/tpcds-data). |
| `compare-table.sh`        | Compare one table's Rust output against the selected reference (`--compat trino` or `--compat c`) via MD5 + diff.               |
| `compare-all-tables.sh`      | Run the full conformance suite for one compat mode (the main CI entry point). Honors per-mode skip lists at the top of the script. |
| `clean-fixtures.sh`       | Remove all generated fixtures under `tests/fixtures/`.                                                                          |

Run any script with `--help` to print its usage block.

---

## Typical Workflow

### Java conformance
```bash
# 1. Generate Java reference fixtures (one-time, or when Java changes).
./scripts/generate-fixtures.sh

# 2. Run the comparison.
./scripts/compare-table.sh <table>     # one table
./scripts/compare-all-tables.sh           # all tables
```

### C dsdgen conformance
```bash
# 1. Download the C reference data (one-time, or to refresh).
./scripts/generate-fixtures.sh --compat c

# 2. Run the comparison in C-compat mode.
./scripts/compare-table.sh <table> --compat c
./scripts/compare-all-tables.sh --compat c
```

### Cleanup
```bash
./scripts/clean-fixtures.sh --yes      # remove all fixtures
```

---

## Requirements

- **Java:** Maven-built TPC-DS JAR at `../tpcds/target/tpcds-*-jar-with-dependencies.jar` (`bootstrap-trino.sh` handles this).
- **C dsdgen reference:** `git`, `tar`, `bzip2` for `generate-fixtures.sh --compat c`. No C compiler required — data is pre-generated.
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
./scripts/generate-fixtures.sh --compat c --scale N
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
- run: ./scripts/bootstrap-trino.sh
- run: ./scripts/generate-fixtures.sh --quiet
- run: ./scripts/compare-all-tables.sh --quiet

# C dsdgen conformance
- run: ./scripts/generate-fixtures.sh --compat c
- run: ./scripts/compare-all-tables.sh --compat c --quiet
```

Exit codes make it easy to fail CI on mismatches.

## Notes

- **Fixtures are gitignored** - They're generated artifacts, not source code
- **Deterministic output** - Same seed always produces same data
- **Byte-for-byte equality** - Not just row count, complete binary match
- **Bug compatibility** - Maintains same quirks as Java/C versions (e.g., leap year bug)
