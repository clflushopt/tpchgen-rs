use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, ShipModeRowGenerator};

pub struct ShipModeArrow {
    generator: ShipModeRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl ShipModeArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::ShipMode);
        Self { generator: ShipModeRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for ShipModeArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for ShipModeArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut sm_sk: Vec<Option<i64>> = Vec::new();
        let mut sm_id: Vec<Option<String>> = Vec::new();
        let mut sm_type: Vec<Option<String>> = Vec::new();
        let mut sm_code: Vec<Option<String>> = Vec::new();
        let mut sm_carrier: Vec<Option<String>> = Vec::new();
        let mut sm_contract: Vec<Option<String>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::ShipMode(r) = g {
                    let nbm = r.null_bit_map();
                    sm_sk.push(sk_opt(nbm, 0, r.get_sm_ship_mode_sk()));
                    sm_id.push(opt(nbm, 1, r.get_sm_ship_mode_id().to_owned()));
                    sm_type.push(opt(nbm, 2, r.get_sm_type().to_owned()));
                    sm_code.push(opt(nbm, 3, r.get_sm_code().to_owned()));
                    sm_carrier.push(opt(nbm, 4, r.get_sm_carrier().to_owned()));
                    sm_contract.push(opt(nbm, 5, r.get_sm_contract().to_owned()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if sm_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(sm_sk)),
            Arc::new(string_view_array_from_opt_iter(sm_id.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(sm_type.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(sm_code.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(sm_carrier.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(sm_contract.iter().map(|s| s.as_deref()))),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("sm_ship_mode_sk", DataType::Int64, true),
    Field::new("sm_ship_mode_id", DataType::Utf8View, true),
    Field::new("sm_type", DataType::Utf8View, true),
    Field::new("sm_code", DataType::Utf8View, true),
    Field::new("sm_carrier", DataType::Utf8View, true),
    Field::new("sm_contract", DataType::Utf8View, true),
])));
