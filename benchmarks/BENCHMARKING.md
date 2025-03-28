# Benchmark Methodology

This directory contains a set of scripts to generate TPCH data at various scale factors


# `TBL` format benchmarks


## tpchgen (`./tbl_tpchgen.sh`)


The CLI from this repo, with default (mutlti-threaded configuration)

Example command for SF=10

```shell
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

# Columnar file formats

## parquet tpchdbgen (`parquet_tpchgen.sh`)
Uses the CLI in this repo. Deafults to snappy compression

This uses less than 400MB of peakmemory as measured by top

Example command for SF=10

```shell
tpchgen-cli -s 10 --format=parquet
```


## duckdb duckdb (`duckdb_duckdb.sh.`)

Note duckdb does not create the `tbl` files, but instead creates a database in its own custom format file so we can not compare directly

We use the TPCH data generator as described here https://duckdb.org/docs/stable/extensions/tpch.html

Note for the larger scale factors, as documneted on https://duckdb.org/docs/stable/extensions/tpch.html, duckdb consumes signifcicant amounts of memoyr

```sql
INSTALL tpch;
LOAD tpch;

.open test
.timer on
CALL dbgen(sf = 1);
```

duckdb test.duckdb "INSTALL tpch; LOAD tpch; CALL dbgen(sf = 1);"

## duckdb duckdb (`parquet_duckdb.sh.`)

Run the above commands for `duckdb` and then export the data to parquet.

note the output parquet file uses SNAPPY compression

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



--- gcp setup


sudo mdadm --create --verbose /dev/md0 --level=0 --raid-devices=4 /dev/nvme1n1 /dev/nvme2n1 /dev/nvme3n1 /dev/nvme4n1
    sudo mkfs -t ext4 /dev/md0
sudo mkdir /data
sudo mount /dev/md0 /data
sudo chmod -R a+rwx /data


alamb@aal-perf:~$ df -h
Filesystem       Size  Used Avail Use% Mounted on
udev              44G     0   44G   0% /dev
tmpfs            8.7G  640K  8.7G   1% /run
/dev/nvme0n1p1   9.7G  7.2G  2.0G  79% /
tmpfs             44G     0   44G   0% /dev/shm
tmpfs            5.0M     0  5.0M   0% /run/lock
/dev/nvme0n1p15  124M   12M  113M  10% /boot/efi
tmpfs            8.7G     0  8.7G   0% /run/user/1000
/dev/md0         1.5T   28K  1.4T   1% /data

cd /data

git clone git@github.com:alamb/tpchgen-rs.git

Can only write at 756MB/s !!!!!

When writing 10gb it goes slightly faster
10737418240 bytes (11 GB, 10 GiB) copied, 13.179 s, 815 MB/s


Install duckdb like this:

curl https://install.duckdb.org | sh

sudo ln -s /home/alamb/.duckdb/cli/latest/duckdb /usr/local/bin
v1.2.1 8e52ec4395

Issues to file / look into:

I can not make SF1000 as I get an error like this:
called `Result::unwrap()` on an `Err` value: General("Parquet does not support more than 32767 row groups per fil\
e (currently: 32768)")

Solutions: make larger row groups / make multiple files perhaps...

--- things I would like to measure:
How long does it take to make SF 10,000 and SF100,000 (and peak memory)

Why is peak memory increasing over time for tpchgen?
