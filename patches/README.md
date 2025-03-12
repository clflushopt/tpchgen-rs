# Apache Trino's TPCH Patches

This is a series of patches that can be used to fixup Apache Trino's TPCH Generator
and have it running locally to generate fixtures for our test suite.

If you'd like an already ready to use version feel free to use [my fork](https://github.com/clflushopt/tpch)
instead.

## Usage

First start by cloning the original repo found [here]http://github.com/trinodb/tpch), then proceed
to apply the `upstream.patch`.

This will create a small `Runner` class that essentially runs the generators and writes the datasets
to CSV files, one file per table. The scale factor and output directory are hardcoded but that can
be easily changed.
