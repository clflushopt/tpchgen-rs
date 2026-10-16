# TPC-DS Generator Benchmarks

[`tpcds_benchmark.sh`](tpcds_benchmark.sh) times TPC-DS data generation with
`tpcgen-cli`, the C reference `dsdgen`, the Trino Java port and DuckDB, and
appends one CSV row per timed step so the results can be charted directly
(for example in Google Sheets). See the header of the script for all options.

| Format  | Tool         | Command                                                      |
|---------|--------------|--------------------------------------------------------------|
| tbl     | `tpcgen-cli` | `tpcgen-cli tpcds dat --num-threads N`                       |
| tbl     | `dsdgen`     | N processes: `dsdgen -PARALLEL N -CHILD i`                   |
| tbl     | `trino`      | `java -jar tpcds.jar --parallelism N`                        |
| parquet | `tpcgen-cli` | `tpcgen-cli tpcds parquet --num-threads N`                   |
| parquet | `duckdb`     | `CALL dsdgen(sf)` into a `.duckdb` file, then `EXPORT DATABASE (FORMAT parquet)` |

`N` defaults to the number of cores so every generator gets the same
parallelism. DuckDB is reported as three rows: `generate`, `convert` and
`total` (the sum).

## Prerequisites

- `tpcgen-cli` (`cargo install tpcgen-cli` or `cargo build --release`)
- `dsdgen` built from the [TPC-DS Tools] (v4.0.0, `make OS=LINUX`); point
  `DSDGEN_DIR` at the `tools/` directory containing `dsdgen` and `tpcds.idx`
- Trino jar: `./scripts/tpcds/bootstrap-trino.sh` (or set `TRINO_JAR`)
- [`duckdb`] CLI on `PATH`
- `/usr/bin/time` (`apt install time` on Debian/Ubuntu)

[TPC-DS Tools]: https://www.tpc.org/tpc_documents_current_versions/current_specifications5.asp
[`duckdb`]: https://duckdb.org/docs/installation/

## Running

```shell
# Everything, SF 1/10/100/300, on a scratch disk
DSDGEN_DIR=~/DSGen-software-code-4.0.0/tools \
  ./tpcgen-cli/benchmarks/tpcds_benchmark.sh --output-dir /data/bench --drop-caches

# Only the parquet comparison at SF 1 and 10, three runs each
./tpcgen-cli/benchmarks/tpcds_benchmark.sh --formats parquet --scale-factors "1 10" --runs 3

# Show the commands without running them
./tpcgen-cli/benchmarks/tpcds_benchmark.sh --dry-run
```

Generated data is deleted after each run unless `--keep` is given; SF 300
needs roughly 300 GB of free disk for the `tbl` runs.

## Output

`tpcds_benchmark.csv` (override with `--results`):

```
timestamp,host,format,tool,version,step,scale_factor,threads,run,wall_sec,user_sec,sys_sec,peak_rss_mb,output_mb,status
2026-10-16T10:50:07Z,bench1,tbl,tpcgen-cli,4.0.0-rc.1,generate,1,16,1,1.19,6.79,0.21,266,1196,ok
2026-10-16T10:50:50Z,bench1,tbl,trino,1.5-SNAPSHOT,generate,1,16,1,43.71,57.04,2.89,2164,1222,ok
2026-10-16T10:51:02Z,bench1,parquet,duckdb,v1.5.6,generate,1,16,1,10.53,10.29,0.24,1234,321,ok
2026-10-16T10:51:02Z,bench1,parquet,duckdb,v1.5.6,convert,1,16,1,0.49,4.35,0.60,2103,371,ok
2026-10-16T10:51:02Z,bench1,parquet,duckdb,v1.5.6,total,1,16,1,11.02,14.64,0.84,2103,371,ok
```

- `peak_rss_mb` is the maximum resident set size reported by
  `/usr/bin/time`. For `dsdgen` it is the largest of the N processes, not
  their sum.
- `status` is `ok`, `fail` (non-zero exit; rows are still written so a
  DuckDB out-of-memory at a large scale factor does not stop the run) or
  `skipped`.
