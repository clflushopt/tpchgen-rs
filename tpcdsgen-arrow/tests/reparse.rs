//! Verifies correctness of the tpcdsgen-arrow generators by reparsing the
//! canonical pipe-delimited .dat format and comparing against the directly
//! generated Arrow RecordBatches.
//!
//! Strategy: for each table, drive the tpcdsgen RowGenerator to produce
//! `Vec<String>` values via `TableRow::get_values()`, write them as
//! pipe-delimited text (one row per line), re-parse with the Arrow CSV reader
//! using the same schema, and assert the two RecordBatches are equal.

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use std::sync::Arc;
use tpcdsgen::config::{Options, Session, Table};
use tpcdsgen::row::{
    CallCenterRowGenerator, CatalogPageRowGenerator, CustomerAddressRowGenerator,
    CustomerDemographicsRowGenerator, CustomerRowGenerator, DateDimRowGenerator,
    GeneratedRow, HouseholdDemographicsRowGenerator,
    IncomeBandRowGenerator, InventoryRowGenerator, ItemRowGenerator, PromotionRowGenerator,
    ReasonRowGenerator, RowGenerator, ShipModeRowGenerator, StoreRowGenerator, TableRow,
    TimeDimRowGenerator, WarehouseRowGenerator, WebPageRowGenerator, WebSiteRowGenerator,
};
use tpcdsgen_arrow::{
    CallCenterArrow, CatalogPageArrow, CustomerAddressArrow, CustomerArrow,
    CustomerDemographicsArrow, DateDimArrow, HouseholdDemographicsArrow,
    IncomeBandArrow, InventoryArrow, ItemArrow, PromotionArrow, ReasonArrow, RecordBatchIterator,
    ShipModeArrow, StoreArrow, TimeDimArrow, WarehouseArrow, WebPageArrow, WebSiteArrow,
};

fn session() -> Session {
    Options::default().to_session().unwrap()
}

/// Write rows as pipe-delimited text and re-parse with the Arrow CSV reader.
fn parse_dat(rows: &[Vec<String>], schema: &SchemaRef) -> RecordBatch {
    let null_re = regex::Regex::new("^$").unwrap();
    let mut data: Vec<u8> = Vec::with_capacity(rows.len() * 64);
    for values in rows {
        data.extend_from_slice(values.join("|").as_bytes());
        data.push(b'\n');
    }
    let builder = arrow_csv::reader::ReaderBuilder::new(Arc::clone(schema))
        .with_batch_size(rows.len().max(1))
        .with_delimiter(b'|')
        .with_header(false)
        .with_null_regex(null_re);
    let mut reader = builder.build(data.as_slice()).unwrap();
    let batch = reader.next().unwrap().unwrap();
    assert!(reader.next().is_none(), "expected exactly one batch from parsed dat");
    batch
}

/// Drive a RowGenerator for `row_count` rows, calling `extract` on each
/// GeneratedRow to collect the string-value vectors.
fn collect_rows<G, F>(mut gen: G, row_count: i64, session: &Session, extract: F) -> Vec<Vec<String>>
where
    G: RowGenerator,
    F: Fn(&GeneratedRow) -> Option<Vec<String>>,
{
    let mut out = Vec::new();
    for row_num in 1..=row_count {
        let result = gen
            .generate_row_and_child_rows(row_num, session, None, None)
            .expect("row gen");
        for g in result.get_rows() {
            if let Some(v) = extract(g) {
                out.push(v);
            }
        }
        gen.consume_remaining_seeds_for_row();
    }
    out
}

/// Core comparison loop: drive the Arrow generator batch by batch, compare
/// each batch against the reparsed pipe-delimited rows.
fn run_test<G, A, F>(gen: G, row_count: i64, session: &Session, mut arrow: A, extract: F)
where
    G: RowGenerator,
    A: RecordBatchIterator,
    F: Fn(&GeneratedRow) -> Option<Vec<String>>,
{
    let all_rows = collect_rows(gen, row_count, session, extract);
    let mut offset = 0;
    while let Some(arrow_batch) = arrow.next() {
        let n = arrow_batch.num_rows();
        let schema = Arc::clone(arrow.schema());
        let reparsed = parse_dat(&all_rows[offset..offset + n], &schema);
        assert_eq!(reparsed, arrow_batch, "batch mismatch at row offset {offset}");
        offset += n;
    }
    assert_eq!(offset, all_rows.len(), "total row count mismatch");
}

// ---------------------------------------------------------------------------
// One test per dimension/simple table.
// Paired fact tables (StoreSales+StoreReturns etc.) are omitted here.
// ---------------------------------------------------------------------------

macro_rules! dim_test {
    ($name:ident, $gen:expr, $arrow:expr, $table:expr, $variant:ident) => {
        #[test]
        fn $name() {
            let s = session();
            let n = s.get_scaling().get_row_count($table);
            run_test(
                $gen,
                n,
                &s,
                $arrow(s.clone()).with_batch_size(512),
                |g| match g {
                    GeneratedRow::$variant(r) => Some(r.get_values()),
                    _ => None,
                },
            );
        }
    };
}

dim_test!(income_band,            IncomeBandRowGenerator::new(),            IncomeBandArrow::new,            Table::IncomeBand,            IncomeBand);
dim_test!(reason,                 ReasonRowGenerator::new(),                ReasonArrow::new,                Table::Reason,                Reason);
dim_test!(ship_mode,              ShipModeRowGenerator::new(),              ShipModeArrow::new,              Table::ShipMode,              ShipMode);
dim_test!(inventory,              InventoryRowGenerator::new(),             InventoryArrow::new,             Table::Inventory,             Inventory);
dim_test!(household_demographics, HouseholdDemographicsRowGenerator::new(), HouseholdDemographicsArrow::new, Table::HouseholdDemographics,  HouseholdDemographics);
dim_test!(customer_demographics,  CustomerDemographicsRowGenerator::new(),  CustomerDemographicsArrow::new,  Table::CustomerDemographics,  CustomerDemographics);
dim_test!(customer_address,       CustomerAddressRowGenerator::new(),       CustomerAddressArrow::new,       Table::CustomerAddress,       CustomerAddress);
dim_test!(customer,               CustomerRowGenerator::new(),              CustomerArrow::new,              Table::Customer,              Customer);
dim_test!(catalog_page,           CatalogPageRowGenerator::new(),           CatalogPageArrow::new,           Table::CatalogPage,           CatalogPage);
dim_test!(time_dim,               TimeDimRowGenerator::new(),               TimeDimArrow::new,               Table::TimeDim,               TimeDim);
dim_test!(date_dim,               DateDimRowGenerator::new(),               DateDimArrow::new,               Table::DateDim,               DateDim);
dim_test!(warehouse,              WarehouseRowGenerator::new(),             WarehouseArrow::new,             Table::Warehouse,             Warehouse);
dim_test!(item,                   ItemRowGenerator::new(),                  ItemArrow::new,                  Table::Item,                  Item);
dim_test!(promotion,              PromotionRowGenerator::new(),             PromotionArrow::new,             Table::Promotion,             Promotion);
dim_test!(store,                  StoreRowGenerator::new(),                 StoreArrow::new,                 Table::Store,                 Store);
dim_test!(web_page,               WebPageRowGenerator::new(),               WebPageArrow::new,               Table::WebPage,               WebPage);
dim_test!(web_site,               WebSiteRowGenerator::new(),               WebSiteArrow::new,               Table::WebSite,               WebSite);
dim_test!(call_center,            CallCenterRowGenerator::new(),            CallCenterArrow::new,            Table::CallCenter,            CallCenter);
