# tpchgen-rs

[![Apache licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[license-badge]: https://img.shields.io/badge/license-Apache%20v2-blue.svg
[license-url]: https://github.com/clflushopt/tpchgen-rs/blob/main/LICENSE
[actions-badge]: https://github.com/clflushopt/tpchgen-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/clflushopt/tpchgen-rs/actions?query=branch%3Amain

Blazing fast TPCH benchmark data generator in pure Rust !

## Usage

`tpchgen-cli` is a [`dbgen`](https://github.com/databricks/tpch-dbgen) compatible CLI tool
that generates tables from the TPCH benchmark dataset.

`tpchgen` is the library that implements the data generation logic for TPCH and it can be
used to embed data generation logic natively in Rust.

## Contributing

Pull requests are welcome. For major changes, please open an issue first for
discussion. See our [contributors guide](CONTRIBUTING.md) for more details.

## License

The project is licensed under the [APACHE 2.0](LICENSE) license.

## References

- The TPC-H Specification, see the specification [page](https://www.tpc.org/tpc_documents_current_versions/current_specifications5.asp).
- The Original `dbgen` Implementation you must submit an official request to access the software `dbgen` at their official [website](https://www.tpc.org/tpch/)
