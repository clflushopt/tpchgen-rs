# Benchmark Methodology

This directory contains a set of scripts to generate TPCH data at various scale factors


# `TBL` format benchmarks


## tpchgen (`./tbl_tpchgen.sh`)


The CLI from this repo, with default (mutlti-threaded configuration)

Example

```shell
# Scale factor 10
tpchgen-cli -s 10
```


## tpchgen1 (`./tbl_tpchgen_1.sh`)

The CLI from this rep, run with `--num-threads=1` to use only a single core

Example

```shell
# Scale factor 10
tpchgen-cli -s 10
```


## `dbgen` (`tbl_dbgen.sh`)

`dbgen` is the classic dbgen program from TPCH. We use an unmodified copy from [electrum/tpch-dbgen](https://github.com/electrum/tpch-dbgen)

Example commands:
```shell
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen
make
./dbgen -vf -s 1
```


## `dbgen_O3` (`tbl_dbgen_O3.sh`)

The `makefile` that comes with the classic dbgen program uses basic
optimization levels (`-O`). A more realistic comparison is to use
maximum optimization (`-O3`).

This variabt runs dbgen with enable realistic optimization levels
```diff
diff --git a/makefile b/makefile
index b72d51a..701c946 100644
--- a/makefile
+++ b/makefile
@@ -110,7 +110,7 @@ DATABASE= ORACLE
MACHINE = MAC
WORKLOAD = TPCH
#
-CFLAGS = -g -DDBNAME=\"dss\" -D$(MACHINE) -D$(DATABASE) -D$(WORKLOAD) -DRNG_TEST -D_FILE_OFFSET_BITS=64
+CFLAGS = -g -DDBNAME=\"dss\" -D$(MACHINE) -D$(DATABASE) -D$(WORKLOAD) -DRNG_TEST -D_FILE_OFFSET_BITS=64  -O3
LDFLAGS = -O
# The OBJ,EXE and LIB macros will need to be changed for compilation under
#  Windows NT
```

## `dbgen_docker` docker (`tbl_dbgen_docker.sh`)

Since compiling `dbgen` is non trivial, projects such as DataFusion (TOOD link) often use a pre-built docker image

Example command to generate scale factor 1 dataset into `/tmp`

```sh
docker run -v "/tmp:/data" --rm ghcr.io/scalytics/tpch-docker:main -vf -s 1
```

# Parquet and similar benchmarks

## tpchgen

## duckdb

Note duckdb does not create the `tbl` files, but instead creates a database file so we can not compare directly

We use the TPCH data generator as described here https://duckdb.org/docs/stable/extensions/tpch.html

```sql
INSTALL tpch;
LOAD tpch;

.open test
.timer on
CALL dbgen(sf = 1);
```

## duckdb (parquet)

Run the above commands for `duckdb` and then export the data to parquet.

```sql
copy customer to 'customer.parquet' (FORMAT parquet);
copy lineitem to 'lineitem.parquet' (FORMAT parquet);
copy nation to 'supplier.parquet' (FORMAT parquet);
copy orders to 'supplier.parquet' (FORMAT parquet);
copy part to 'part.parquet' (FORMAT parquet);
copy partsupp to 'partsupp.parquet' (FORMAT parquet);
copy region to 'region.parquet' (FORMAT parquet);
copy supplier to 'supplier.parquet' (FORMAT parquet);
```
