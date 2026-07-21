//! [`TpcdsGenerationPlan`]: how a TPC-DS table is split into chunks that can
//! be generated in parallel.

use std::ops::RangeInclusive;
use tpcdsgen::config::{Scaling, Table};

/// Parquet files can have at most 32767 row groups
const MAX_ROW_GROUPS: i64 = 32767;

/// What a chunk of a table is generated into.
///
/// Selects the estimated output bytes per source row, which is what turns a
/// target chunk size in bytes into a number of source rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChunkFormat {
    /// One chunk is one Parquet row group. The number of chunks is capped at
    /// Parquet's row group limit.
    Parquet,
    /// One chunk is one in memory buffer of DAT text.
    Dat,
}

/// How to generate a TPC-DS table: a list of contiguous source row ranges,
/// each of which is generated as one chunk (a Parquet row group, or a buffer
/// of DAT text). Each range can be generated independently, in parallel.
///
/// The number of chunks is computed from the source row count, an estimated
/// output size per source row, and the target chunk size.
///
/// Note the ranges are over *source* rows, which is not the same as output rows
/// for all tables: for example, the sales generators emit several output rows
/// per source row, and the returns tables are generated from their paired sales
/// generator, so their ranges are over the *sales* source rows.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TpcdsGenerationPlan {
    /// Inclusive 1-based source row ranges, one per chunk
    ranges: Vec<RangeInclusive<i64>>,
}

impl TpcdsGenerationPlan {
    /// Compute the chunk layout for `table` given the target `chunk_bytes` of
    /// generated `format` output.
    pub(super) fn new(
        table: Table,
        scaling: &Scaling,
        chunk_bytes: usize,
        format: ChunkFormat,
    ) -> Self {
        let source_rows = scaling.get_row_count(table.source_table());
        let estimated_bytes =
            source_rows.saturating_mul(estimated_bytes_per_source_row(table, format));
        let max_chunks = match format {
            ChunkFormat::Parquet => MAX_ROW_GROUPS,
            // DAT chunks are just buffers, so there is no limit on how many
            // there can be. Capping them would instead grow the buffers, and
            // with them the peak memory use, at high scale factors.
            ChunkFormat::Dat => i64::MAX,
        };
        let num_chunks = (estimated_bytes / chunk_bytes.max(1) as i64 + 1)
            .min(max_chunks)
            .min(source_rows)
            .max(1);
        // ceiling division so the last chunk is the one that comes up short
        let rows_per_chunk = ((source_rows + num_chunks - 1) / num_chunks).max(1);

        let mut ranges = Vec::with_capacity(num_chunks as usize);
        let mut start = 1;
        while start <= source_rows {
            let end = (start + rows_per_chunk - 1).min(source_rows);
            ranges.push(start..=end);
            start = end + 1;
        }
        // An empty table still needs one (empty) range so that an (empty)
        // output file is written: for Parquet that is a valid file containing
        // the table schema, for DAT an empty file.
        if ranges.is_empty() {
            #[allow(clippy::reversed_empty_ranges)]
            ranges.push(1..=0);
        }
        Self { ranges }
    }

    /// Return the number of chunks this plan will generate
    pub(super) fn chunk_count(&self) -> usize {
        self.ranges.len()
    }
}

/// Converts the plan into an iterator of inclusive source row ranges
impl IntoIterator for TpcdsGenerationPlan {
    type Item = RangeInclusive<i64>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

/// Estimated output bytes written per *source* row (see
/// [`TpcdsGenerationPlan`] for what a source row is).
fn estimated_bytes_per_source_row(table: Table, format: ChunkFormat) -> i64 {
    match format {
        ChunkFormat::Parquet => estimated_parquet_bytes_per_source_row(table),
        ChunkFormat::Dat => estimated_dat_bytes_per_source_row(table),
    }
}

/// Estimated (uncompressed) Parquet bytes written per *source* row (see
/// [`TpcdsGenerationPlan`] for what a source row is).
///
/// Row group sizes are conventionally measured in uncompressed bytes, which
/// is also what the previous `ArrowWriter` based implementation limited.
///
/// Measured from files generated at scale factor 1: the total uncompressed
/// bytes, computed using datafusion-cli:
///
/// You can verify these numbers using
/// ```shell
/// for table in call_center catalog_page catalog_returns catalog_sales customer customer_address \
///   customer_demographics date_dim dbgen_version household_demographics income_band inventory \
///   item promotion reason ship_mode store store_returns store_sales time_dim warehouse web_page \
///   web_returns web_sales web_site; do
///   case "$table" in
///     catalog_sales|catalog_returns)
///       source_rows="(select count(distinct cs_order_number) from 'catalog_sales.parquet')"
///       ;;
///     store_sales|store_returns)
///       source_rows="(select count(distinct ss_ticket_number) from 'store_sales.parquet')"
///       ;;
///     web_sales|web_returns)
///       source_rows="(select count(distinct ws_order_number) from 'web_sales.parquet')"
///       ;;
///     *)
///       source_rows="(select count(*) from '$table.parquet')"
///       ;;
///   esac
///
///   datafusion-cli -q -c "
///   select
///     '$table' as table_name,
///     round(
///       cast(sum(total_uncompressed_size) as double) / cast($source_rows as double)
///     ) as bytes_per_source_row
///   from parquet_metadata('$table.parquet')"
/// done
/// ```
///
/// Which results in something like
/// ```text
/// +-------------+----------------------+
/// | table_name  | bytes_per_source_row |
/// +-------------+----------------------+
/// | call_center | 423.0                |
/// +-------------+----------------------+
/// ...
/// +-----------------+----------------------+
/// | table_name      | bytes_per_source_row |
/// +-----------------+----------------------+
/// | catalog_returns | 195.0                |
/// +-----------------+----------------------+
/// +---------------+----------------------+
/// | table_name    | bytes_per_source_row |
/// +---------------+----------------------+
/// | catalog_sales | 2391.0               |
/// +---------------+----------------------+
/// ```
///
/// Remember you have to divide by the **source** row count (which is different
/// for sales vs returns tables) to get the bytes per source row.
fn estimated_parquet_bytes_per_source_row(table: Table) -> i64 {
    match table {
        Table::CallCenter => 423,
        Table::CatalogPage => 113,
        Table::CatalogReturns => 195,
        Table::CatalogSales => 2391,
        Table::Customer => 92,
        Table::CustomerAddress => 46,
        Table::CustomerDemographics => 9,
        Table::DateDim => 57,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 358,
        Table::HouseholdDemographics => 10,
        Table::IncomeBand => 20,
        Table::Inventory => 3,
        Table::Item => 197,
        Table::Promotion => 120,
        Table::Reason => 54,
        Table::ShipMode => 76,
        Table::Store => 265,
        Table::StoreReturns => 220,
        Table::StoreSales => 2366,
        Table::TimeDim => 38,
        Table::Warehouse => 206,
        Table::WebPage => 50,
        Table::WebReturns => 261,
        Table::WebSales => 3119,
        Table::WebSite => 218,
        // Not a main table; never generated as Parquet output
        _ => unreachable!("Parquet generation plans are only defined for main TPC-DS tables"),
    }
}

/// Estimated DAT bytes written per *source* row (see [`TpcdsGenerationPlan`]
/// for what a source row is).
///
/// Measured from files generated at scale factor 1: the size of each
/// `<table>.dat` file divided by the source row count of the table
/// (`Scaling::get_row_count(table.source_table())`), rounded to whole bytes.
///
/// DAT rows are fixed width in the sense that matters here: the field values
/// come from the same distributions at every scale factor, so bytes per source
/// row does not change with the scale factor.
fn estimated_dat_bytes_per_source_row(table: Table) -> i64 {
    match table {
        Table::CallCenter => 315,
        Table::CatalogPage => 139,
        Table::CatalogReturns => 134,
        Table::CatalogSales => 1849,
        Table::Customer => 132,
        Table::CustomerAddress => 110,
        Table::CustomerDemographics => 42,
        Table::DateDim => 141,
        // Note: this value is not performance critical as this is a 1 row table
        // and the size depends on the command line args.
        Table::DbgenVersion => 210,
        Table::HouseholdDemographics => 21,
        Table::IncomeBand => 16,
        Table::Inventory => 20,
        Table::Item => 281,
        Table::Promotion => 124,
        Table::Reason => 38,
        Table::ShipMode => 56,
        Table::Store => 263,
        Table::StoreReturns => 136,
        Table::StoreSales => 1619,
        Table::TimeDim => 59,
        Table::Warehouse => 117,
        Table::WebPage => 96,
        Table::WebReturns => 163,
        Table::WebSales => 2448,
        Table::WebSite => 292,
        // Not a main table; never generated as DAT output
        _ => unreachable!("DAT generation plans are only defined for main TPC-DS tables"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_ROW_GROUP_BYTES: usize = 7 * 1024 * 1024;

    fn plan(table: Table, scale_factor: f64, row_group_bytes: usize) -> TpcdsGenerationPlan {
        TpcdsGenerationPlan::new(
            table,
            &Scaling::new(scale_factor),
            row_group_bytes,
            ChunkFormat::Parquet,
        )
    }

    fn dat_plan(table: Table, scale_factor: f64, chunk_bytes: usize) -> TpcdsGenerationPlan {
        TpcdsGenerationPlan::new(
            table,
            &Scaling::new(scale_factor),
            chunk_bytes,
            ChunkFormat::Dat,
        )
    }

    /// Assert the ranges cover `1..=expected_source_rows` contiguously
    fn assert_covers(plan: &TpcdsGenerationPlan, expected_source_rows: i64) {
        let mut next_row = 1;
        for range in &plan.ranges {
            assert_eq!(*range.start(), next_row);
            assert!(range.end() >= range.start());
            next_row = range.end() + 1;
        }
        assert_eq!(next_row, expected_source_rows + 1);
    }

    #[test]
    fn small_table_single_row_group() {
        let plan = plan(Table::Reason, 1.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.ranges, vec![1..=35]);
    }

    #[test]
    fn store_sales_sf1_default() {
        let plan = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // ~568 MB estimated output in 7 MB row groups over 240k source rows
        assert_eq!(plan.chunk_count(), 78);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn store_returns_ranges_use_sales_source_rows() {
        let plan = plan(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES);
        // store_returns is generated from the 240k store_sales source rows
        // (its own scaling row count is 0)
        assert_eq!(plan.chunk_count(), 8);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn smaller_row_groups_make_more_row_groups() {
        let default = plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        let small = plan(Table::StoreSales, 1.0, 1024 * 1024);
        assert!(small.chunk_count() > default.chunk_count());
        assert_covers(&small, 240_000);
    }

    #[test]
    fn row_group_count_is_capped() {
        let plan = plan(Table::StoreSales, 3000.0, 1024);
        // ceiling division can leave the count just under the cap
        assert!(plan.chunk_count() <= MAX_ROW_GROUPS as usize);
        assert!(plan.chunk_count() > (MAX_ROW_GROUPS - 2) as usize);
        let source_rows = Scaling::new(3000.0).get_row_count(Table::StoreSales);
        assert_covers(&plan, source_rows);
    }

    #[test]
    fn row_groups_never_exceed_source_rows() {
        // 35 source rows in 1 byte row groups still yields at most 35 groups
        let plan = plan(Table::Reason, 1.0, 1);
        assert_eq!(plan.chunk_count(), 35);
        assert_covers(&plan, 35);
    }

    #[test]
    fn empty_table_gets_one_empty_range() {
        let plan = plan(Table::Reason, 0.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.chunk_count(), 1);
        assert!(plan.ranges[0].is_empty());
    }

    #[test]
    fn dat_chunks_are_sized_from_dat_bytes() {
        // store_sales DAT output is ~1619 bytes per source row: ~389 MB at SF 1
        let plan = dat_plan(Table::StoreSales, 1.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.chunk_count(), 53);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn dat_chunk_count_is_not_capped_at_the_parquet_row_group_limit() {
        // DAT chunks are buffers, so they keep their target size rather than
        // growing to stay under Parquet's row group limit
        let plan = dat_plan(Table::StoreSales, 3000.0, DEFAULT_ROW_GROUP_BYTES);
        assert!(plan.chunk_count() > MAX_ROW_GROUPS as usize);
        assert_covers(&plan, Scaling::new(3000.0).get_row_count(Table::StoreSales));
    }

    #[test]
    fn dat_returns_ranges_use_sales_source_rows() {
        let plan = dat_plan(Table::StoreReturns, 1.0, DEFAULT_ROW_GROUP_BYTES);
        assert_covers(&plan, 240_000);
    }

    #[test]
    fn dat_empty_table_gets_one_empty_range() {
        let plan = dat_plan(Table::Reason, 0.0, DEFAULT_ROW_GROUP_BYTES);
        assert_eq!(plan.chunk_count(), 1);
        assert!(plan.ranges[0].is_empty());
    }
}
