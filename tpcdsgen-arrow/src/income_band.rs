use crate::conversions::opt;
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, RecordBatch};
use arrow::error::ArrowError;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, IncomeBandRowGenerator, RowGenerator};

pub struct IncomeBandArrow {
    generator: IncomeBandRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl IncomeBandArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::IncomeBand);
        Self { generator: IncomeBandRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchIterator for IncomeBandArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for IncomeBandArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Result<RecordBatch, ArrowError>> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut band_id: Vec<Option<i32>> = Vec::new();
        let mut lower: Vec<Option<i32>> = Vec::new();
        let mut upper: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::IncomeBand(r) = g {
                    let nbm = r.null_bit_map();
                    band_id.push(opt(nbm, 0, r.get_ib_income_band_id()));
                    lower.push(opt(nbm, 1, r.get_ib_lower_bound()));
                    upper.push(opt(nbm, 2, r.get_ib_upper_bound()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if band_id.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int32Array::from(band_id)),
            Arc::new(Int32Array::from(lower)),
            Arc::new(Int32Array::from(upper)),
        ]))
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
    Field::new("ib_income_band_id", DataType::Int32, false),
    Field::new("ib_lower_bound", DataType::Int32, false),
    Field::new("ib_upper_bound", DataType::Int32, false),
]))
}
