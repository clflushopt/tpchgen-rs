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

## Try now!

First [install Rust](https://www.rust-lang.org/tools/install) and this tool:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install tpchgen-cli
```

```shell
# create Scale Factor 10 (3.6GB, 8 files, 60M rows in lineitem) in 5 seconds on a modern laptop
tpchgen-cli -s 10 --format=parquet
```

## Performance

| Scale Factor | `tpchgen-cli` | DuckDB     | DuckDB (proprietary) |
|--------------|---------------|------------|----------------------|
| 1            | `0:02.24`     | `0:12.29`  | `0:10.68`            |
| 10           | `0:09.97`     | `1:46.80`  | `1:41.14`            |
| 100          | `1:14.22`     | `17:48.27` | `16:40.88`           |
| 1000         | `10:26.26`    | N/A (OOM)  | N/A (OOM)            |           

* DuckDB (proprietary) is the time required to create TPCH data using the
  proprietary DuckDB format
* Creating Scale Factor 1000 data in DuckDB [requires 647 GB of memory], 
  which is why it is not included in the table above. 

[required 647 GB of memory]: https://duckdb.org/docs/stable/extensions/tpch.html#resource-usage-of-the-data-generator

Times to create TPCH tables in Parquet format using `tpchgen-cli` and `duckdb` for various scale factors.

![Parquet Generation Performance](parquet-performance.png)

[`tpchgen-cli`](tpchgen-cli/README.md) is more than 10x faster than the next
fastest TPCH generator we know of. On a 2023 Mac M3 Max laptop, it easily
generates data faster than can be written to SSD. See
[BENCHMARKS.md](benchmarks/BENCHMARKS.md) for more details on performance and
benchmarking.

## Testing

This crate has extensive tests to ensure correctness. We compare the output of
this crate with the original `dbgen` implementation as part of every checkin.
See [TESTING.md](TESTING.md) for more details.

## Crates

- `tpchgen` is the library that implements the data generation logic for TPCH
  and it can be used to embed data generation logic natively in Rust.

- `tpchgen-arrow` is a library for generating in memory [Apache Arrow]
  record batches for each of the TPCH tables.

- `tpchgen-cli` is a [`dbgen`](https://github.com/databricks/tpch-dbgen)
  compatible CLI tool that generates tables from the TPCH benchmark dataset.

[Apache Arrow]: https://arrow.apache.org/

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
