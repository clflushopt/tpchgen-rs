# TPCH Prepared Test Files

This folder stores test files used to ensure consistency with Apache Trino's and
OLTPBenchmarks outputs.

The files are stored as gzipped CSV files and we plan to potentially add support
for parquet in the futures.

## CSV Test Files

The folders follow the `sf-{scale-factor}` pattern.

| Folder   | Description                             |
| -------- | --------------------------------------- |
| sf-0.01  | TPCH dataset of a scale factor of 0.01  |
| sf-0.001 | TPCH dataset of a scale factor of 0.001 |

The CSV files are all named after the tables they represent.

| File         | Description         |
| ------------ | ------------------- |
| parts.csv    | TPCH parts table    |
| customer.csv | TPCH customer table |
| lineitem.csv | TPCH linetime table |
| nation.csv   | TPCH nation table   |
| orders.csv   | TPCH order table    |
| partsupp.csv | TPCH partsupp table |
| region.csv   | TPCH region table   |
| supplier.csv | TPCH supplier table |

## The TPCH schema

```
+-----------------+        +-------------------+       +--------------------+       +-------------------+
| PART (P_)       |        | PARTSUPP (PS_)    |       | LINEITEM (L_)      |       | ORDERS (O_)       |
| SF*200,000      |        | SF*800,000        |       | SF*6,000,000       |       | SF*1,500,000      |
+-----------------+        +-------------------+       +--------------------+       +-------------------+
| PARTKEY    PK   |------->| PARTKEY      FK   |----+  | ORDERKEY      FK   |<------| ORDERKEY      PK  |
| NAME            |   +--->| SUPPKEY      FK   |--+ +->| PARTKEY       FK   |   +-->| CUSTKEY       FK  |
| MFGR            |   |    | AVAILQTY          |  +--->| SUPPKEY       FK   |   |   | ORDERSTATUS       |
| BRAND           |   |    | SUPPLYCOST        |       | LINENUMBER         |   |   | TOTALPRICE        |
| TYPE            |   |    | COMMENT           |       | QUANTITY           |   |   | ORDERDATE         |
| SIZE            |   |    +-------------------+       | EXTENDEDPRICE      |   |   | ORDERPRIORITY     |
| CONTAINER       |   |                                | DISCOUNT           |   |   | CLERK             |
| RETAILPRICE     |   |                                | TAX                |   |   | SHIPPRIORITY      |
| COMMENT         |   |                                | RETURNFLAG         |   |   | COMMENT           |
+-----------------+   |                                | LINESTATUS         |   |   +-------------------+
                      |                                | SHIPDATE           |   |           ^
+-----------------+   |    +-------------------+       | COMMITDATE         |   |           |
| SUPPLIER (S_)   |   |    | CUSTOMER (C_)     |       | RECEIPTDATE        |   |           |
| SF*10,000       |   |    | SF*150,000        |       | SHIPINSTRUCT       |   |           |
+-----------------+   |    +-------------------+       | SHIPMODE           |   |           |
| SUPPKEY    PK   |---.    | CUSTKEY     PK    |---+-->| COMMENT            |   |           |
| NAME            |   |    | NAME              |   |   +--------------------+   |           |
| ADDRESS         |   |    | ADDRESS           |   +----------------------------+           |
| NATIONKEY  FK   |---+--->| NATIONKEY    FK   |--------------------------------------------+
| PHONE           |        | PHONE             |
| ACCTBAL         |        | ACCTBAL           |
| COMMENT         |        | MKTSEGMENT        |
+-----------------+        | COMMENT           |
         ^                 +-------------------+
         |                         |
         |                         v
+-----------------+       +-------------------+
| NATION (N_)     |       | REGION (R_)       |
| 25              |       | 5                 |
+-----------------+       +-------------------+
| NATIONKEY  PK   |       | REGIONKEY    PK   |
| NAME            |       | NAME              |
| REGIONKEY  FK   |------>| COMMENT           |
| COMMENT         |       +-------------------+
+-----------------+
```
