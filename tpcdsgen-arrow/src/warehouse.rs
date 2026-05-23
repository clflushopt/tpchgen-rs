use crate::conversions::{address_columns, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, WarehouseRowGenerator};

pub struct WarehouseArrow {
    generator: WarehouseRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl WarehouseArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Warehouse);
        Self { generator: WarehouseRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for WarehouseArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for WarehouseArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut w_sk: Vec<Option<i64>> = Vec::new();
        let mut w_id: Vec<Option<String>> = Vec::new();
        let mut w_name: Vec<Option<String>> = Vec::new();
        let mut w_sq_ft: Vec<Option<i32>> = Vec::new();
        let mut addr_rows: Vec<(tpcdsgen::types::Address, i64, u32)> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::Warehouse(r) = g {
                    let nbm = r.null_bit_map();
                    w_sk.push(sk_opt(nbm, 0, r.get_w_warehouse_sk()));
                    w_id.push(opt(nbm, 1, r.get_w_warehouse_id().to_owned()));
                    w_name.push(opt(nbm, 2, r.get_w_warehouse_name().to_owned()));
                    w_sq_ft.push(opt(nbm, 3, r.get_w_warehouse_sq_ft()));
                    addr_rows.push((r.get_w_address().clone(), nbm, 4));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if w_sk.is_empty() { return None; }

        let (street_number, street_name, street_type, suite_number, city, county, state, zip, country, gmt_offset) =
            address_columns(addr_rows.iter().map(|(a, nbm, base)| (a, *nbm, *base)));

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(w_sk)),
            Arc::new(string_view_array_from_opt_iter(w_id.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(w_name.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(w_sq_ft)),
            Arc::new(street_number),
            Arc::new(street_name),
            Arc::new(street_type),
            Arc::new(suite_number),
            Arc::new(city),
            Arc::new(county),
            Arc::new(state),
            Arc::new(zip),
            Arc::new(country),
            Arc::new(gmt_offset),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("w_warehouse_sk", DataType::Int64, true),
    Field::new("w_warehouse_id", DataType::Utf8View, true),
    Field::new("w_warehouse_name", DataType::Utf8View, true),
    Field::new("w_warehouse_sq_ft", DataType::Int32, true),
    Field::new("w_street_number", DataType::Int32, true),
    Field::new("w_street_name", DataType::Utf8View, true),
    Field::new("w_street_type", DataType::Utf8View, true),
    Field::new("w_suite_number", DataType::Utf8View, true),
    Field::new("w_city", DataType::Utf8View, true),
    Field::new("w_county", DataType::Utf8View, true),
    Field::new("w_state", DataType::Utf8View, true),
    Field::new("w_zip", DataType::Utf8View, true),
    Field::new("w_country", DataType::Utf8View, true),
    Field::new("w_gmt_offset", DataType::Int32, true),
])));
