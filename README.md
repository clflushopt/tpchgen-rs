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

### CLI Usage

We tried to make the `tpchgen-cli` experience as close to `dbgen` as possible for no other
reason than maybe make it easier for you to have a drop-in replacement.

```sh
$ tpchgen-cli -h
TPC-H Data Generator

Usage: tpchgen-cli [OPTIONS] --output-dir <OUTPUT_DIR>

Options:
  -s, --scale-factor <SCALE_FACTOR>  Scale factor to address defaults to 1 [default: 1]
  -o, --output-dir <OUTPUT_DIR>      Output directory for generated files
  -t, --tables <TABLES>              Which tables to generate (default: all) [possible values: nation, region, part, supplier, part-supp, customer, orders, line-item]
  -p, --parts <PARTS>                Number of parts to generate (for parallel generation) [default: 1]
      --part <PART>                  Which part to generate (1-based, only relevant if parts > 1) [default: 1]
  -h, --help                         Print help
```

For example generating a dataset with a scale factor of 1 (1GB) can be done like this :

```sh
$ tpchgen-cli -s 1 --output-dir=/tmp/tpch
```

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

