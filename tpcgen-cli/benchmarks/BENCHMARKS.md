# TPC-DS Generator Benchmarks

This document contains a benchmarking results and methodology for TPC-DS data
generation.

Performance (default, all 8 cores)

![TPC-DS tbl generation time](tpcds-tbl-performance.png)

![TPC-DS Parquet generation time](tpcds-parquet-performance.png)

Performance (1-core) *

(TODO IMAGE)

# Experimental Setup:

All measurements were gathered on a `c8gn.2xlarge` instance (8 vCPU, 16GB RAM,
aarch64 / Graviton) from AWS. For storage, I used 2 gp3 EBS 1100 GB volumes,
configured in RAID0, with 2000 mb/s provisioned throughput with 4000 IOPS.

Note that the storage configuration is critical. `tpcgen-cli` generates data
around 1.2 GB/s on a `c8gn.2xlarge`, well above the default throughput limit of
250MB/sec on `gp3`. I had to use two volumes because even a single volume
configured at "2000 mb/s" still left the CPUs stalled on I/O.

The volumes were attached as `/dev/sdb` and `/dev/sdc` and formatted as a RAID0
array with `mdadm` and an ext4 filesystem:

```shell
sudo mdadm --create --verbose /dev/md0 --level=0 --raid-devices=2 /dev/sdb /dev/sdc
sudo mkfs -t ext4 /data
sudo mount /dev/md0 /data
```

I ran the benchmark script with the following command:
```shell
DSDGEN_DIR=/home/ec2-user/DSGen-software-code-4.0.0/tools/ ./tpcds_benchmark.sh  --output-dir /data/bench-data
```

Single core performance was measured with:
```shell
DSDGEN_DIR=/home/ec2-user/DSGen-software-code-4.0.0/tools/ ./tpcds_benchmark.sh  --output-dir /data/bench-data --threads 1 
````

```shell
DSDGEN_DIR=/home/ec2-user/DSGen-software-code-4.0.0/tools/ ./tpcds_benchmark.sh  --output-dir /data/bench-data
[13:17:50] host=ip-172-31-58-244 threads=8 scale_factors=[1 10 100 300] tools=tpcgen-cli,dsdgen,trino,duckdb formats=tbl,parquet runs=1
[13:17:50] versions: tpcgen-cli=4.0.0-rc.1 dsdgen=4.0.0 trino=1.5-SNAPSHOT duckdb=v1.5.6
[13:17:50] results -> ./tpcds_benchmark.csv, data -> /data/bench-data
```

We compared 4 generators:
* `tpcgen-cli` (Rust) -- this repo
* `dsdgen` (C) -- the official TPC-DS generator (version 4.0)
* `trino` (Java) -- the Trino port of the TPC-DS generator (version 1.5-SNAPSHOT)
* `duckdb` (C++) -- the DuckDB port of the TPC-DS generator (version 1.5.6)

Here are the commands used to run each generator:

| Format  | Tool         | Command                                                      |
|---------|--------------|--------------------------------------------------------------|
| tbl     | `tpcgen-cli` | `tpcgen-cli tpcds dat --num-threads N`                       |
| tbl     | `dsdgen`     | N processes: `dsdgen -PARALLEL N -CHILD i`                   |
| tbl     | `trino`      | `java -jar tpcds.jar --parallelism N`                        |
| parquet | `tpcgen-cli` | `tpcgen-cli tpcds parquet --num-threads N`                   |
| parquet | `duckdb`     | `CALL dsdgen(sf)` into a `.duckdb` file, then `EXPORT DATABASE (FORMAT parquet)` |


Generate the plots with
```shell
uv run plot_tpcds_benchmark.py tpcds_benchmark.csv --subtitle "AWS c8gn.2xlarge (8 vCPU, 16 GB RAM)"
```