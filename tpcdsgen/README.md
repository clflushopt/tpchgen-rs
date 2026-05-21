# TPC-DS Data Generator Crate

This crate provides the core data generator logic for TPC-H.

## Usage

```bash
# Build the generator
cargo build --release

# Generate all tables at scale factor 1 (default)
./target/release/tpcdsgen

# Generate all tables at scale factor 10
./target/release/tpcdsgen --scale 10

# Generate specific table
./target/release/tpcdsgen --table store_sales --scale 10

# Generate to a specific directory
./target/release/tpcdsgen --scale 10 --directory /path/to/output
```

## Generating Fixtures

Fixtures are pre-generated TPC-DS data files used for conformance testing.

### Directory Structure

```
tests/fixtures/
├── scale-1-trino/    # Java reference fixtures (`--compat trino`)
├── scale-1-c/       # C dsdgen reference fixtures (`--compat c`)
└── scale-10-trino/   # higher scale factors as needed
```

### Conformance Testing

`tpcdsgen` ships with two conformance suites, both implemented as shell
scripts that do byte-for-byte (MD5) comparison of `.dat` output. See
[scripts/README.md](scripts/README.md) for full details.

**vs. Java / Trino reference (default, `--compat trino`):**

```bash
# Default (MD5-only): no Java setup required.
./scripts/compare-all-tables.sh --scale 1

# Byte-for-byte (--full): one-time Java setup + fixture generation.
./scripts/bootstrap-trino.sh
./scripts/generate-fixtures.sh
./scripts/compare-all-tables.sh --scale 1 --full
```

**vs. C dsdgen reference (`--compat c`):**

```bash
# Default (MD5-only): no download needed.
./scripts/compare-all-tables.sh --compat c --scale 1

# Byte-for-byte (--full): one-time data download from
# https://github.com/alamb/tpcds-data into tests/fixtures/scale-N-c/.
./scripts/generate-fixtures.sh --compat c --scale 1
./scripts/compare-all-tables.sh --compat c --scale 1 --full
```

Both suites also support comparing a single table:

```bash
./scripts/compare-table.sh reason                       # MD5-only, vs. Java
./scripts/compare-table.sh reason --compat c            # MD5-only, vs. C dsdgen
./scripts/compare-table.sh reason --full                # byte-for-byte, vs. Java
```

### Verifying Fixtures with MD5SUMS

Each fixture directory contains an `MD5SUMS` file for verification.

**On Linux:**
```bash
cd tests/fixtures/scale-1-trino
md5sum -c MD5SUMS
```

**On macOS:**
```bash
cd tests/fixtures/scale-1-trino
while read hash file; do
  [[ $(md5 -q "$file") == "$hash" ]] && echo "$file: OK" || echo "$file: FAILED"
done < MD5SUMS
```

## Known Bugs

The TPC-DS reference implementation contains several bugs that must be replicated for benchmark compliance.
These bugs originated in the C implementation and were faithfully reproduced in the Java port. Our Rust implementation
also replicates these bugs to ensure byte-for-byte compatibility with the reference implementation.

See [BUGS.md](BUGS.md) for a detailed list of documented bugs, more will be added.


## TPC-DS Reference MD5 Hashes

These are the canonical MD5 hashes the Rust implementation is verified
against. They are committed alongside the test fixtures so the
conformance scripts can do an MD5-only check without downloading or
re-generating the full reference data.

### Java / Trino reference (`--compat trino`)

Generated locally from `java -jar tpcds-*.jar --scale N`:

- Scale 1:  [`tests/fixtures/scale-1-trino/MD5SUMS`](tests/fixtures/scale-1-trino/MD5SUMS)
- Scale 10: [`tests/fixtures/scale-10-trino/MD5SUMS`](tests/fixtures/scale-10-trino/MD5SUMS)

### C `dsdgen` reference (`--compat c`)

Sourced from [alamb/tpcds-data](https://github.com/alamb/tpcds-data#md5-checksums):

- Scale 1:  [`tests/fixtures/scale-1-c/MD5SUMS`](tests/fixtures/scale-1-c/MD5SUMS)
- Scale 2:  [`tests/fixtures/scale-2-c/MD5SUMS`](tests/fixtures/scale-2-c/MD5SUMS)
- Scale 5:  [`tests/fixtures/scale-5-c/MD5SUMS`](tests/fixtures/scale-5-c/MD5SUMS)
- Scale 10: [`tests/fixtures/scale-10-c/MD5SUMS`](tests/fixtures/scale-10-c/MD5SUMS)

`dbgen_version.dat` contains a generation timestamp and will differ
between runs; the conformance suite excludes it from comparison.

## Verification

To verify the Rust implementation matches:

```bash
# Verify at scale 1
./scripts/compare-all-tables.sh --scale 1

# Verify at scale 10
./scripts/compare-all-tables.sh --scale 10
```
