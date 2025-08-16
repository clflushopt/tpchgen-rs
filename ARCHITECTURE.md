# Architecture Guide

## Crate Organization
The project is organized into two crates:

1. `tpchgen`: The core library that implements the data generation logic for TPCH.
2. `tpchgen-arrow`: Generates the TPCH data directly as the [Apache Arrow](https://arrow.apache.org/) in memory format
3. `tpchgen-cli`: A CLI tool that uses the `tpchgen` library to generate TPCH data.

## Dependencies

The `tpchgen` crate is designed to be embeddable in as many locations as
possible and thus has no dependencies by design. For example, it does
not depend on arrow or parquet crates or display libraries.

`tpchgen-arrow` is similarly designe to be embeddable with minimal dependencies
and only depends on the [`arrow` crate](https://docs.rs/arrow)

The `tpchgen-cli` crate is designed to include many useful features, and thus
has many more dependencies.

## Performance

Speed is a very important aspect of this project, and care has been taken to keep 
the code as fast as possible, using some of the following techniques:
1. Avoiding heap allocations during data generation
2. Integer arithmetic and display instead of floating point arithmetic and display
3. Using multiple cores and tuned buffer sizes
