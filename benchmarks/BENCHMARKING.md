# Benchmark Methodology

## `tpchgen-rs`

The CLI from this rep

## `dbgen` (native)

We use the copy of tpch-dbgen from here:https://github.com/electrum/tpch-dbgen
```shell
git clone https://github.com/electrum/tpch-dbgen.git
cd tpch-dbgen
make
./dbgen -vf -s 1
````

## `dbgen` (native -O3)

Apply this patch to the makefile to enable realistic optimization levels
```diff
andrewlamb@Andrews-MacBook-Pro-2:~/Software/tpch-dbgen$ git diff -b
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

## `dbgen` (docker)

Generate a scale factor 1 dataset into `/tmp`

```sh
docker run -v "/tmp:/data" --rm ghcr.io/scalytics/tpch-docker:main -vf -s 1
```

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
