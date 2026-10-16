# TPC-DS Generator Benchmarks

This document describes how we  benchmarked TPC-DS data generation.

Performance(mutli-core) 
(TODO RESULTS)

Performance (single-core) *


Note that `trino`, `dsdgen` and `duckdb` use a single core for data generation
(despite CLI args that imply they use multiple threads). `tpcgen-cli` uses all
available cores by default, but can be limited with `--num-threads N`, so we
also report single core performance.

# Experimental Setup:

All measurements were gathered using an AWS `c8gn.2xlarge` instance (8 vCPU,
16GB RAM, aarch64 / Graviton):

`/data` is 2 1100 GB gp3 volumes in RAID0, each configured with 2000 mb/s
provisioned throughput with 4000 iops (the minimum allowed for gp3). 

The storage configuration is critical because `tpcgen-cli` generates data
at over 1.2GB/s, well above the default 250MB/sec gp3 default.

The data is written to `/data`, a RAID0 of 2 gp3 volumes, ext4 filesystem:
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
