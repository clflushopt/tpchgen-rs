#!/bin/bash
#
# Runs the duckdb to generate parquet at various scale factors

set -x
set -e

LOGFILE=parquet_duckdb.txt
echo "***********Timings**********" >> $LOGFILE
uname -a >> $LOGFILE

SCALE_FACTORS="1 10 100"
rm -rf out_duckdb
mkdir out_duckdb
for sf in $SCALE_FACTORS ; do
    echo "SF=$sf" >> $LOGFILE
    /usr/bin/time -a -o $LOGFILE duckdb out_duckdb/$sf.duckdb "\
INSTALL tpch;\
LOAD tpch;\
CALL dbgen(sf = $sf);\
copy customer to 'out_duckdb/customer.parquet' (FORMAT parquet);\
copy lineitem to 'out_duckdb/lineitem.parquet' (FORMAT parquet);\
copy nation   to 'out_duckdb/nation.parquet' (FORMAT parquet);\
copy orders   to 'out_duckdb/orders.parquet' (FORMAT parquet);\
copy part     to 'out_duckdb/part.parquet'     (FORMAT parquet);\
copy partsupp to 'out_duckdb/partsupp.parquet' (FORMAT parquet);\
copy region   to 'out_duckdb/region.parquet'   (FORMAT parquet);\
copy supplier to 'out_duckdb/supplier.parquet' (FORMAT parquet);"

done
