# tpchgen-rs

TPC-H benchmark data generator in pure Rust !

## Usage

`tpchgen-rs` is a CLI tool that generates the TPC-H benchmark dataset and is
compatible with DuckDB's `dbgen` extension.

For example the following command generates 1GB worth of data of the specified
tables and writes it as Parquet files to the specified directory.

**THIS IS MOSTLY WHAT I THINK THE API WILL END UP LOOKING LIKE**

```rust
tpchgen --scale 1 --tables "p,c,s,o" --output ./dir
```

`tpchgen` is the library that implements the data generation logic for TPC-H
it can be used to extend or embed data generation logic.

**THIS IS MOSTLY WHAT I THINK THE API WILL END UP LOOKING LIKE**

```rust

use tpchgen::Generators::tablegen;

fn main() {
    let generator = tablegen!("part", "customer", "supplier", "orders", "nation");
    let scale = 1;
    let tables = generator.generate(scale);

    tables.iter().for_each().rows().for_each(|row| println!("{}", row));
}

```

## Contributing

Pull requests are welcome. For major changes, please open an issue first for
discussion.

## License

The project is licensed under the [APACHE 2.0](LICENSE) license.

## References

- The TPC-H Specification, see the specification [page](https://www.tpc.org/tpc_documents_current_versions/current_specifications5.asp).
- The Original `dbgen` Implementation you must submit an official request to access the software `dbgen` at their official [website](https://www.tpc.org/tpch/)
