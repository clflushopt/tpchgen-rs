# Architecture Guide

## Crate Organization
The project is organized into two crates:

1. `tpchgen`: The core library that implements the data generation logic for TPCH.
2. `tpchgen-cli`: A CLI tool that uses the `tpchgen` library to generate TPCH data.

## Dependencies

The `tpchgen` crate is designed to be embeddable in as many locations as
possible and thus has very minimal dependencies by design. For example, it does
not depend on arrow or parquet crates or display libraries.

The `tpchgen-cli` crate is designed to include many useful features, and thus
has many more dependencies.

## Speed

Speed is a very important aspect of this project, and care has been taken to keep 
the code as fast as possible, using some of the following techniques:
1. Avoiding heap allocations during data generation
2. Integer arithmetic and display instead of floating point arithmetic and display
