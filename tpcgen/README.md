# TPC Data Generator CLI

`tpcgen` is a command line interface for generating TPC-H and TPC-DS benchmark data.

## Try with `uvx`

```shell
uvx tpcgen tpch -s 1 --output-dir /tmp/tpch
```

## Install with `pip`

```shell
python -m pip install tpcgen
```

## Examples

```shell
tpcgen tpch -s 1 --output-dir /tmp/tpch
tpcgen tpch csv -s 1 --output-dir /tmp/tpch
tpcgen tpch parquet -s 100 --tables lineitem --parts 10 --output-dir /tmp/tpch
```
