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
