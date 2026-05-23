use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, ReasonRowGenerator, RowGenerator};

pub struct ReasonArrow {
    generator: ReasonRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl ReasonArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Reason);
        Self { generator: ReasonRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchIterator for ReasonArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for ReasonArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut sk: Vec<Option<i64>> = Vec::new();
        let mut id: Vec<Option<String>> = Vec::new();
        let mut desc: Vec<Option<String>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::Reason(r) = g {
                    let nbm = r.null_bit_map();
                    sk.push(sk_opt(nbm, 0, r.get_r_reason_sk()));
                    id.push(opt(nbm, 1, r.get_r_reason_id().to_owned()));
                    desc.push(opt(nbm, 2, r.get_r_reason_description().to_owned()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(sk)),
            Arc::new(string_view_array_from_opt_iter(id.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(desc.iter().map(|s| s.as_deref()))),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("r_reason_sk", DataType::Int64, true),
    Field::new("r_reason_id", DataType::Utf8View, true),
    Field::new("r_reason_description", DataType::Utf8View, true),
])));
