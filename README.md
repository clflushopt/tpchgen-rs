# tpchgen-rs

[![Apache licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[license-badge]: https://img.shields.io/badge/license-Apache%20v2-blue.svg
[license-url]: https://github.com/clflushopt/tpchgen-rs/blob/main/LICENSE
[actions-badge]: https://github.com/clflushopt/tpchgen-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/clflushopt/tpchgen-rs/actions?query=branch%3Amain

Blazing fast [TPCH] benchmark data generator, in pure Rust with zero dependencies.

[TPCH]: https://www.tpc.org/tpch/

## Features

1. Blazing Speed 🚀
2. Obsessively Tested 📋
3. Fully parallel, streaming, constant memory usage 🧠

## Try it now!

### Install Using Python
Install this tool with Python:
```shell
pip install tpchgen-cli
```

```shell
# create Scale Factor 10 (3.6GB, 8 files, 60M rows in lineitem) in 5 seconds on a modern laptop
tpchgen-cli -s 10 --format=parquet
```

### Install Using Rust
[Install Rust](https://www.rust-lang.org/tools/install) and this tool:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tpchgen-cli
```

```shell
# create Scale Factor 10 (3.6GB, 8 files, 60M rows in lineitem) in 5 seconds on a modern laptop
tpchgen-cli -s 10 --format=parquet
```

Or watch this [awesome demo](https://www.youtube.com/watch?v=UYIC57hlL14) recorded by [@alamb](https://github.com/alamb)
and the companion blog post in the [Datafusion blog](https://datafusion.apache.org/blog/2025/04/10/fastest-tpch-generator/).

### Examples

```shell

# Create a scale factor 10 dataset in the native table format.
tpchgen-cli -s 10 --output-dir sf10

# Create a scale factor 1 dataset in Parquet format.
tpchgen-cli -s 1 --output-dir sf1-parquet --format=parquet

# Create a scale factor 1 (default) partitioned dataset for the region, nation, orders
# and customer tables.
tpchgen-cli --tables region,nation,orders,customer --output-dir sf1-partitioned --parts 10 --part 2

# Create a scale factor 1 partitioned into separate folders.
#
# Each folder will have a single partition of rows, the partition size will depend on the scale
# factor. For tables that have less rows than the minimum partition size like "nation" or "region"
# the generator will produce the same file in each part.
#
# $ md5sum part-*/{nation,region}.tbl
# 2f588e0b7fa72939b498c2abecd9fbbe  part-1/nation.tbl
# 2f588e0b7fa72939b498c2abecd9fbbe  part-2/nation.tbl
# c235841b00d29ad4f817771fcc851207  part-1/region.tbl
# c235841b00d29ad4f817771fcc851207  part-2/region.tbl
for PART in `seq 1 2`; do
  mkdir part-$PART
  tpchgen-cli --tables region,nation,orders,customer --output-dir part-$PART --parts 10 --part $PART
done
```

## Performance

| Scale Factor | `tpchgen-cli` | DuckDB     | DuckDB (proprietary) |
| ------------ | ------------- | ---------- | -------------------- |
| 1            | `0:02.24`     | `0:12.29`  | `0:10.68`            |
| 10           | `0:09.97`     | `1:46.80`  | `1:41.14`            |
| 100          | `1:14.22`     | `17:48.27` | `16:40.88`           |
| 1000         | `10:26.26`    | N/A (OOM)  | N/A (OOM)            |

- DuckDB (proprietary) is the time required to create TPCH data using the
  proprietary DuckDB format
- Creating Scale Factor 1000 data in DuckDB [requires 647 GB of memory],
  which is why it is not included in the table above.

[required 647 GB of memory]: https://duckdb.org/docs/stable/extensions/tpch.html#resource-usage-of-the-data-generator

Times to create TPCH tables in Parquet format using `tpchgen-cli` and `duckdb` for various scale factors.

![Parquet Generation Performance](parquet-performance.png)

[`tpchgen-cli`](./tpchgen-cli/README.md) is more than 10x faster than the next
fastest TPCH generator we know of. On a 2023 Mac M3 Max laptop, it easily
generates data faster than can be written to SSD. See
[BENCHMARKS.md](./benchmarks/BENCHMARKS.md) for more details on performance and
benchmarking.

## Testing

This crate has extensive tests to ensure correctness and produces exactly the
same, byte-for-byte output as the original [`dbgen`] implementation. We compare
the output of this crate with [`dbgen`] as part of every checkin. See
[TESTING.md](TESTING.md) for more details on testing methodology

## Crates

- [`tpchgen`](tpchgen): the core data generator logic for TPC-H. It has no
  dependencies and is easy to embed in other Rust project. 

- [`tpchgen-arrow`](tpchgen-arrow) generates TPC-H data in [Apache Arrow]
  format. It depends on the arrow-rs library

- [`tpchgen-cli`](tpchgen-cli) is a [`dbgen`] compatible CLI tool that generates
  benchmark dataset using multiple processes.

[Apache Arrow]: https://arrow.apache.org/
[`dbgen`]: https://github.com/electrum/tpch-dbgen

## Contributing

Pull requests are welcome. For major changes, please open an issue first for
discussion. See our [contributors guide](CONTRIBUTING.md) for more details.

## Architecture

Please see [architecture guide](ARCHITECTURE.md) for details on how the code
is structured.

## License

The project is licensed under the [APACHE 2.0](LICENSE) license.

## References

- The TPC-H Specification, see the specification [page](https://www.tpc.org/tpc_documents_current_versions/current_specifications5.asp).
- The Original `dbgen` Implementation you must submit an official request to access the software `dbgen` at their official [website](https://www.tpc.org/tpch/)
