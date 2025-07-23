//! [`GenerationPlan`] that describes how to generate a TPC-H dataset.

use crate::{OutputFormat, Table};
use log::debug;
use std::fmt::Display;
use tpchgen::generators::{
    CustomerGenerator, OrderGenerator, PartGenerator, PartSuppGenerator, SupplierGenerator,
};

/// A list of generator "parts" (data generator chunks, not TPCH parts)
///
/// Controls the parallelization and layout of Parquet files in `tpchgen-cli`.
///
/// # Background
///
/// A "part" is a logical partition of a particular output table. Each data
/// generator can create parts individually.
///
/// For example, the parameters to [`OrderGenerator::new`] `scale_factor,
/// `part_count` and `part_count` together define a partition of the `Order`
/// table.
///
/// The entire output table results from generating each of the `part_count` parts. For
/// example, if `part_count` is 10, appending parts 1 to 10 results in a
/// complete `Order` table.
///
/// Interesting properties of parts:
/// 1. They are independent of each other, so they can be generated in parallel.
/// 2. They scale. So for example, parts `0..10` with a `part_count` of 50
///    will generate the same data as parts `1` with a `part_count` of 5.
///
/// # Implication for tpchgen-cli
///
/// For `tbl` and `csv` files, tpchgen-cli generates `num-threads` parts in
/// parallel.
///
/// For Parquet files, the output file has one row group for each "part".
///
/// # Example
/// ```
/// let plan = GenerationPlan::new(
///   &Table::Orders,
///   OutputFormat::Parquet,
///   1.0, // scale factor
///   -1, // cli_part
///   -1, // cli_parts
///  );
/// let results = plan.into_iter().collect::<Vec<_>>();
/// /// assert_eq!(results.len(), 1);
/// ```
#[derive(Debug)]
pub struct GenerationPlan {
    /// Total number of parts to generate
    part_count: i32,
    /// List of parts
    part_list: Vec<i32>,
}

impl GenerationPlan {
    /// Returns the number of parts to generate
    ///
    /// cli_part and cli_part_count are passed from the CLI arguments
    /// to specify a particular part or number of parts to generate.
    pub fn new(
        table: &Table,
        format: OutputFormat,
        scale_factor: f64,
        cli_part: i32,
        cli_part_count: i32,
        num_threads: usize,
    ) -> Self {
        // If a single part is specified, split it into chunks to enable parallel generation.
        if cli_part != -1 || cli_part_count != -1 {
            // These tables are small not parameterized by part count,
            // so we must create only a single part.
            if table == &Table::Nation || table == &Table::Region {
                return Self {
                    part_count: 1,
                    part_list: vec![1],
                };
            }

            // sanity check arguments (TODO: real Errors)
            if cli_part < 1 || cli_part_count < 1 || cli_part > cli_part_count {
                panic!(
                    "Invalid CLI part or part count. \
                    Expect greater than 1 and cli_part <= cli_part_count. \
                    Got: cli_part={cli_part}, cli_part_count={cli_part_count}",
                );
            }

            let num_chunks = num_threads as i32;

            // The new total number of parts is the original number of parts multiplied by the number of chunks.
            let new_total_parts = cli_part_count * num_chunks;

            // The new part numbers to generate are the chunks that make up the original part.
            let start_part = (cli_part - 1) * num_chunks + 1;
            let end_part = cli_part * num_chunks;
            let new_parts_to_generate = (start_part..=end_part).collect();
            debug!(
                "Generating {} parts for table {:?} with scale factor {}",
                new_total_parts, table, scale_factor
            );
            debug!(
                "CLI part: {}, CLI part count: {}, num_threads: {}",
                cli_part, cli_part_count, num_threads
            );
            debug!("New parts to generate: {:?}", new_parts_to_generate);
            return Self {
                part_count: new_total_parts,
                part_list: new_parts_to_generate,
            };
        }

        // Note use part=1, part_count=1 to calculate the total row count
        // for the table
        //
        // Avg row size is an estimate of the average row size in bytes from the first 100 rows
        // of the table in tbl format
        let (avg_row_size_bytes, row_count) = match table {
            Table::Nation => (88, 1),
            Table::Region => (77, 1),
            Table::Part => (115, PartGenerator::calculate_row_count(scale_factor, 1, 1)),
            Table::Supplier => (
                140,
                SupplierGenerator::calculate_row_count(scale_factor, 1, 1),
            ),
            Table::Partsupp => (
                148,
                PartSuppGenerator::calculate_row_count(scale_factor, 1, 1),
            ),
            Table::Customer => (
                160,
                CustomerGenerator::calculate_row_count(scale_factor, 1, 1),
            ),
            Table::Orders => (114, OrderGenerator::calculate_row_count(scale_factor, 1, 1)),
            Table::Lineitem => {
                // there are on average 4 line items per order.
                // For example, in SF=10,
                // * orders has 15,000,000 rows
                // * lineitem has around 60,000,000 rows
                let row_count = 4 * OrderGenerator::calculate_row_count(scale_factor, 1, 1);
                (128, row_count)
            }
        };
        // target chunks of about 16MB (use 15MB to ensure we don't exceed the target size)
        let target_chunk_size_bytes = 15 * 1024 * 1024;
        let mut num_parts = ((row_count * avg_row_size_bytes) / target_chunk_size_bytes) + 1;

        // parquet files can have at most 32767 row groups so cap the number of parts at that number
        if format == OutputFormat::Parquet {
            num_parts = num_parts.min(32767);
        }

        // convert to i32
        let num_parts = num_parts.try_into().unwrap();
        // generating all the parts

        Self {
            part_count: num_parts,
            part_list: (1..=num_parts).collect(),
        }
    }
}

/// Converts the `GenerationPlan` into an iterator of (part_number, num_parts)
impl IntoIterator for GenerationPlan {
    type Item = (i32, i32);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.part_list
            .into_iter()
            .map(|part_number| (part_number, self.part_count))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Display for GenerationPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenerationPlan for {} parts", self.part_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sf1_nation() {
        Test::new()
            .with_table(Table::Nation)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(1, [1])
    }

    #[test]
    fn sf1_region() {
        Test::new()
            .with_table(Table::Region)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(1, [1])
    }

    #[test]
    fn sf1_part() {
        Test::new()
            .with_table(Table::Part)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(2, [1, 2])
    }

    #[test]
    fn sf1_supplier() {
        Test::new()
            .with_table(Table::Supplier)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(1, [1])
    }

    #[test]
    fn sf1_partsupp() {
        Test::new()
            .with_table(Table::Partsupp)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(2, [1, 2])
    }

    #[test]
    fn sf1_customer() {
        Test::new()
            .with_table(Table::Customer)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(2, 1..=2)
    }

    #[test]
    fn sf1_orders() {
        Test::new()
            .with_table(Table::Orders)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(11, 1..=11)
    }

    #[test]
    fn sf1_lineitem() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .assert(49, 1..=49)
    }

    #[test]
    fn sf1_nation_cli_parts() {
        Test::new()
            .with_table(Table::Nation)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            // nation table is small, so it can not be made in parts
            .with_cli_part(1)
            .with_cli_part_count(10)
            // we expect there is still only one part
            .assert(1, [1])
    }

    #[test]
    fn sf1_region_cli_parts() {
        Test::new()
            .with_table(Table::Region)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            // region table is small, so it can not be made in parts
            .with_cli_part(1)
            .with_cli_part_count(10)
            // we expect there is still only one part
            .assert(1, [1])
    }

    #[test]
    fn sf1_lineitem_cli_parts_1() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            // Generate only part 1 of the lineitem table
            .with_cli_part(1)
            .with_cli_part_count(10)
            // we expect there are num_threads * 10 parts
            .assert(40, [1, 2, 3, 4])
    }

    #[test]
    fn sf1_lineitem_cli_parts_4() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .with_cli_part(4) // part 4 of 10
            .with_cli_part_count(10)
            // we expect there are num_threads * 10 parts
            .assert(40, [13, 14, 15, 16])
    }

    #[test]
    fn sf1_lineitem_cli_parts_10() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .with_cli_part(10) // part 10 of 10
            .with_cli_part_count(10)
            // we expect there are num_threads * 10 parts
            .assert(40, [37, 38, 39, 40])
    }

    #[test]
    #[should_panic(
        expected = "Invalid CLI part or part count. Expect greater than 1 and cli_part <= cli_part_count. Got: cli_part=0, cli_part_count=10"
    )]
    fn sf1_lineitem_cli_parts_invalid_small() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .with_cli_part(0) // part 0 of 10 (invalid)
            .with_cli_part_count(10)
            .assert(40, [13, 14, 15, 16])
    }

    #[test]
    #[should_panic(
        expected = "Invalid CLI part or part count. Expect greater than 1 and cli_part <= cli_part_count. Got: cli_part=11, cli_part_count=10"
    )]
    fn sf1_lineitem_cli_parts_invalid_big() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1.0)
            .with_cli_part(11) // part 11 of 10 (invalid)
            .with_cli_part_count(10)
            .assert(40, [13, 14, 15, 16])
    }

    #[test]
    fn sf10_lineitem_parquet_limit() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Parquet)
            .with_scale_factor(10.0)
            .assert(489, 1..=489);
    }

    #[test]
    fn sf10_lineitem_tbl_limit() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(10.0)
            .assert(489, 1..=489);
    }

    #[test]
    fn sf1000_lineitem_tbl_limit() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Tbl)
            .with_scale_factor(1000.0)
            .assert(48829, 1..=48829);
    }

    #[test]
    fn sf1000_lineitem_parquet_limit() {
        Test::new()
            .with_table(Table::Lineitem)
            .with_format(OutputFormat::Parquet)
            .with_scale_factor(1000.0)
            .assert(32767, 1..=32767);
    }

    /// Test fixture for [`GenerationPlan`].
    #[derive(Debug)]
    struct Test {
        table: Table,
        format: OutputFormat,
        scale_factor: f64,
        cli_part: i32,
        cli_part_count: i32,
        num_cpus: usize,
    }

    impl Test {
        fn new() -> Self {
            Default::default()
        }

        /// Create a [`GenerationPlan`] and assert it has the
        /// expected number of parts and part numbers.
        fn assert(
            self,
            expected_part_count: i32,
            expected_part_numbers: impl IntoIterator<Item = i32>,
        ) {
            let plan = GenerationPlan::new(
                &self.table,
                self.format,
                self.scale_factor,
                self.cli_part,
                self.cli_part_count,
                self.num_cpus,
            );
            assert_eq!(plan.part_count, expected_part_count);
            let expected_part_numbers: Vec<i32> = expected_part_numbers.into_iter().collect();
            assert_eq!(plan.part_list, expected_part_numbers);
        }

        /// Set table
        fn with_table(mut self, table: Table) -> Self {
            self.table = table;
            self
        }

        /// Set output format
        fn with_format(mut self, format: OutputFormat) -> Self {
            self.format = format;
            self
        }

        /// Set scale factor
        fn with_scale_factor(mut self, scale_factor: f64) -> Self {
            self.scale_factor = scale_factor;
            self
        }

        /// Set CLI part
        fn with_cli_part(mut self, cli_part: i32) -> Self {
            self.cli_part = cli_part;
            self
        }

        /// Set CLI part count
        fn with_cli_part_count(mut self, cli_part_count: i32) -> Self {
            self.cli_part_count = cli_part_count;
            self
        }
    }

    impl Default for Test {
        fn default() -> Self {
            Self {
                table: Table::Orders,
                format: OutputFormat::Tbl,
                scale_factor: 1.0,
                cli_part: -1,
                cli_part_count: -1,
                num_cpus: 4, // hard code 4 cores for testing
            }
        }
    }
}
