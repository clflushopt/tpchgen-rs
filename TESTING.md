# Testing

This repository checks its generators against the original reference
implementations: `dbgen` for TPC-H, and both the C `dsdgen` and the Trino Java
port for TPC-DS. Comparisons are byte-for-byte rather than statistical.

Only the text formats (`.tbl` for TPC-H, `.dat` for TPC-DS) can be compared to a
reference implementation directly, because those are the only formats the
reference implementations produce. Arrow, CSV, and Parquet are verified against
that text output instead, so their correctness follows from its.

## The chain of trust

```
  C dbgen (TPC-H)     --.
  C dsdgen (TPC-DS)   --+--> .tbl / .dat --> Arrow --+--> CSV
  Trino Java (TPC-DS) --'                            '--> Parquet
```

### Text output against the reference implementations

`tpcgen-cli tpch tbl` and `tpcgen-cli tpcds dat` are compared to reference data
produced by the original generators. Comparison is by MD5 of the whole file,
with an optional row-level `diff` when a hash does not match. The expected
hashes live in `tpcgen-cli/tests/fixtures/` and are committed to the repo, so
the common case needs neither a container runtime nor a reference build.

The suites live next to the unified CLI crate:

- TPC-H: [tpcgen-cli/scripts/tpch/](tpcgen-cli/scripts/tpch/). `compare-all-tables.sh`
  checks output against `tpcgen-cli/tests/fixtures/tpch/scale-N/MD5SUMS`,
  generated from C `dbgen` by `generate-fixtures.sh` (which needs docker or
  podman).
- TPC-DS: [tpcgen-cli/scripts/tpcds/](tpcgen-cli/scripts/tpcds/). Two reference
  implementations, selected with `--compat trino` (the Java port the Rust code
  was derived from) and `--compat c` (the TPC-supplied C `dsdgen`, whose
  pre-generated data comes from [alamb/tpcds-data]). See the suite's
  [README](tpcgen-cli/scripts/tpcds/README.md) for details, and
  [tpcdsgen/BUGS.md](tpcdsgen/BUGS.md) for the Java bugs that `--compat c`
  corrects.

```sh
./tpcgen-cli/scripts/tpch/compare-all-tables.sh --scale 1
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 1
./tpcgen-cli/scripts/tpcds/compare-all-tables.sh --scale 1 --compat c
```

TPC-H has a second, finer-grained check at the library level:
`tpchgen/tests/integration_tests.rs` compares each generator row by row against
gzipped reference tables checked into `tpchgen/data/` at scale factors 0.001 and
0.01, so a failure points at a single record rather than a file hash.

### Arrow against the text output

`tpchgen-arrow/tests/reparse.rs` and `tpcdsgen-arrow/tests/reparse.rs` write
rows out through the same `Display` and CSV formatting impls the CLI uses, read
them back with the Arrow CSV reader, and assert the result equals the
`RecordBatch`es the Arrow generators produce directly. Since the text output is
known to match the reference implementations, matching it transitively makes the
Arrow output correct as well.

The TPC-DS version covers all 24 conformance tables. It also restarts each
generator partway through the table (see `skip_starting_row`) to check that a
generator resumed at an arbitrary source row produces the same rows as one run
from the start, which is what parallel generation relies on.

### CSV against Arrow

The same reparse tests run a second time over CSV output, using
`tpcdsgen::csv::{csv_header, GeneratedRowCsv}` for TPC-DS and the `*Csv` row
wrappers in `tpchgen::csv` for TPC-H. These are the types
`tpcgen-cli/src/tpcds_cli/csv.rs` and its TPC-H counterpart use to write real
CSV files, so the test exercises the shipped formatting code rather than a
parallel implementation of it.

### Parquet against Arrow

Parquet is written from the Arrow batches, so the CLI integration tests in
`tpcgen-cli/tests/cli_integration/` check the properties the writer could break
on its own:

- Splitting a table across row groups generated from separate source row ranges
  gives the same data as a single pass (`test_tpcgen_cli_tpcds_parquet_matches_single_pass_generation`).
  `store_returns` is generated from the `store_sales` generator, so this also
  proves no return rows are lost or duplicated at range boundaries.
- `--num-threads 1` and `--num-threads 4` produce byte-identical files.
- The Arrow schema survives the round trip, including types with no exact
  Parquet equivalent such as the `Time32(Second)` column in `dbgen_version`.

## Conformance in CI

Conformance runs in two tiers.

Every pull request runs `tpch-conformance.yml` and `tpcds-conformance.yml`.
These do the MD5-only check against the committed `MD5SUMS`, which proves the
output is byte-identical to what the recorded reference hashes describe. TPC-H
covers scale factors 0.001, 0.01, 0.1 and 1; TPC-DS covers scale factor 1
against both compat modes. `rust.yml` runs `cargo nextest run --workspace` on
the same trigger, which is where the reparse and CLI integration tests run.

Every merge to main runs `full-conformance.yml`, the slow pass. It rebuilds the
reference data from the reference implementations themselves (C `dbgen` in
docker, the published C `dsdgen` dataset, and a fresh Maven build of the Trino
Java port), diffs our output against it byte for byte, and fails if the
committed `MD5SUMS` have drifted from the living reference.

## Known coverage limits

The suites do not cover everything:

- `dbgen_version` is excluded from TPC-DS conformance. It records a creation
  timestamp and the command line used, so it can never match a reference file.
  Its schema and command-line handling are covered by CLI integration tests
  instead.
- The TPC-DS reparse tests cap at 10,000 source rows per table
  (`MAX_REPARSE_SOURCE_ROWS`) at scale factor 1, to keep the suite fast. Large
  fact tables are verified over that prefix plus a mid-table restart point, not
  end to end.
- Parquet is compared against Arrow for `store_sales` and `store_returns` only.
  Other tables rely on the writer path being shared.
- `MD5SUMS` are committed for `scale-10-trino`, `scale-5-c` and `scale-10-c`, but
  no workflow currently checks them. CI verifies TPC-DS at scale factor 1
  (both compat modes) and, in the full pass, scale factor 2 for C.
- The reparse tests run in Trino compat mode only, since that is
  `Session::default()`. The `--compat c` corrections are covered at the `.dat`
  level but not through the Arrow, CSV and Parquet paths.
- The step from text back to Arrow uses the `arrow-csv` reader as its parser, so
  a bug in that reader that exactly mirrored a bug in our Arrow generators would
  not be caught.

## Verifying it yourself

`tpcgen-cli tpch` generates exactly the same bytes as the original `dbgen`
program, which you can confirm with `shasum`:

```sh
$ shasum /tmp/sf10/lineitem.tbl tpch-dbgen/lineitem.tbl
c3f5d0218b6623125887d7021922d1e91da11613  /tmp/sf10/lineitem.tbl
c3f5d0218b6623125887d7021922d1e91da11613  tpch-dbgen/lineitem.tbl
```

The checksums below are for the tables `dbgen` itself produces, recorded here
because generating them takes a long time. They come from the [`tpch-dbgen`]
repo, built and run on macOS:

```shell
git clone https://github.com/electrum/tpch-dbgen
cd tpch-dbgen
make
./dbgen -s <scale_factor>
cd ..
shasum tpch-dbgen/*.tbl
```

[`tpch-dbgen`]: https://github.com/electrum/tpch-dbgen
[alamb/tpcds-data]: https://github.com/alamb/tpcds-data

### Scale factor 1

```sh
$ shasum tpch-dbgen/*.tbl
bee45e9c240e87d63786324696b1babad18a5e0b  tpch-dbgen/customer.tbl
4802b21c9975d965aa11893214a879df0d8d9e01  tpch-dbgen/lineitem.tbl
f361dffd3d927f5aa64e71cff91458fb5ea1315f  tpch-dbgen/nation.tbl
00d790a08a116feec992cea14272a4f1e5c55925  tpch-dbgen/orders.tbl
06615f7433806c06162af49c7bc27166c64a31d6  tpch-dbgen/part.tbl
db0fcb935904765a9085505b5feb5260752f8bf3  tpch-dbgen/partsupp.tbl
ac61de9604337e791f1bdbcef8f0cdcc21b01514  tpch-dbgen/region.tbl
baad047476a2720d99b707b6f7a7c9e50c170d5a  tpch-dbgen/supplier.tbl
```

### Scale factor 10

```sh
$ shasum tpch-dbgen/*.tbl
b717482bde38c8312cf232e7ca73aab62f5e1eca  tpch-dbgen/customer.tbl
c3f5d0218b6623125887d7021922d1e91da11613  tpch-dbgen/lineitem.tbl
f361dffd3d927f5aa64e71cff91458fb5ea1315f  tpch-dbgen/nation.tbl
dddffc12e235da9cd8d17584dc1eab237654cb0f  tpch-dbgen/orders.tbl
efb2a169b6ce80d8ed3989147e8d70e7f2a38d6c  tpch-dbgen/part.tbl
eae140257dc91ba3b4a929c32ebe3d08f3605618  tpch-dbgen/partsupp.tbl
ac61de9604337e791f1bdbcef8f0cdcc21b01514  tpch-dbgen/region.tbl
42a76ba965916326e52adca1725ed9ee18b8e61b  tpch-dbgen/supplier.tbl
```

### Scale factor 100

```sh
$ shasum tpch-dbgen/*.tbl
18f5a1784d3adbd4662c35ed1d98897a9773a0dc  tpch-dbgen/customer.tbl
d5a3d8a3ccf7bb20d4ff5f01589b5004504907ec  tpch-dbgen/lineitem.tbl
f361dffd3d927f5aa64e71cff91458fb5ea1315f  tpch-dbgen/nation.tbl
dbfe1ff7481a8e1c2deeba00a0b36c3efb093d0b  tpch-dbgen/orders.tbl
f6eb11ed8a2b4d7d70e30b334fc4fc5a28e03ea4  tpch-dbgen/part.tbl
0d9070467528371790f43e1a6463358ddfcd5f62  tpch-dbgen/partsupp.tbl
ac61de9604337e791f1bdbcef8f0cdcc21b01514  tpch-dbgen/region.tbl
48bc62481b58ff96e5e50a70b3892f4d95f7372f  tpch-dbgen/supplier.tbl
```
