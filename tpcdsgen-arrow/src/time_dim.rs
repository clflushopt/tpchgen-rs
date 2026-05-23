use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, TimeDimRowGenerator};

pub struct TimeDimArrow {
    generator: TimeDimRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl TimeDimArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::TimeDim);
        Self { generator: TimeDimRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for TimeDimArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for TimeDimArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut t_sk: Vec<Option<i64>> = Vec::new();
        let mut t_id: Vec<Option<String>> = Vec::new();
        let mut t_time: Vec<Option<i32>> = Vec::new();
        let mut t_hour: Vec<Option<i32>> = Vec::new();
        let mut t_minute: Vec<Option<i32>> = Vec::new();
        let mut t_second: Vec<Option<i32>> = Vec::new();
        let mut t_am_pm: Vec<Option<String>> = Vec::new();
        let mut t_shift: Vec<Option<String>> = Vec::new();
        let mut t_sub_shift: Vec<Option<String>> = Vec::new();
        let mut t_meal_time: Vec<Option<String>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::TimeDim(r) = g {
                    let nbm = r.null_bit_map();
                    t_sk.push(sk_opt(nbm, 0, r.t_time_sk));
                    t_id.push(opt(nbm, 1, r.t_time_id.clone()));
                    t_time.push(opt(nbm, 2, r.t_time));
                    t_hour.push(opt(nbm, 3, r.t_hour));
                    t_minute.push(opt(nbm, 4, r.t_minute));
                    t_second.push(opt(nbm, 5, r.t_second));
                    t_am_pm.push(opt(nbm, 6, r.t_am_pm.clone()));
                    t_shift.push(opt(nbm, 7, r.t_shift.clone()));
                    t_sub_shift.push(opt(nbm, 8, r.t_sub_shift.clone()));
                    t_meal_time.push(opt(nbm, 9, r.t_meal_time.clone()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if t_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(t_sk)),
            Arc::new(string_view_array_from_opt_iter(t_id.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(t_time)),
            Arc::new(Int32Array::from(t_hour)),
            Arc::new(Int32Array::from(t_minute)),
            Arc::new(Int32Array::from(t_second)),
            Arc::new(string_view_array_from_opt_iter(t_am_pm.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(t_shift.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(t_sub_shift.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(t_meal_time.iter().map(|s| s.as_deref()))),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("t_time_sk", DataType::Int64, false),
    Field::new("t_time_id", DataType::Utf8View, false),
    Field::new("t_time", DataType::Int32, false),
    Field::new("t_hour", DataType::Int32, false),
    Field::new("t_minute", DataType::Int32, false),
    Field::new("t_second", DataType::Int32, false),
    Field::new("t_am_pm", DataType::Utf8View, false),
    Field::new("t_shift", DataType::Utf8View, false),
    Field::new("t_sub_shift", DataType::Utf8View, false),
    Field::new("t_meal_time", DataType::Utf8View, true),
])));
