use crate::conversions::{opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::RecordBatch;
use arrow::error::ArrowError;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DbgenVersionRowGenerator, GeneratedRow, RowGenerator};

pub struct DbgenVersionArrow {
    generator: DbgenVersionRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl DbgenVersionArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DbgenVersion);
        Self { generator: DbgenVersionRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for DbgenVersionArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for DbgenVersionArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Result<RecordBatch, ArrowError>> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut version: Vec<Option<String>> = Vec::new();
        let mut create_date: Vec<Option<String>> = Vec::new();
        let mut create_time: Vec<Option<String>> = Vec::new();
        let mut cmdline: Vec<Option<String>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::DbgenVersion(r) = g {
                    let nbm = r.null_bit_map();
                    version.push(opt(nbm, 0, r.get_dv_version().to_owned()));
                    create_date.push(opt(nbm, 1, r.get_dv_create_date().to_owned()));
                    create_time.push(opt(nbm, 2, r.get_dv_create_time().to_owned()));
                    cmdline.push(opt(nbm, 3, r.get_dv_cmdline_args().to_owned()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if version.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(string_view_array_from_opt_iter(version.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(create_date.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(create_time.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(cmdline.iter().map(|s| s.as_deref()))),
        ]))
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
    Field::new("dv_version", DataType::Utf8View, true),
    Field::new("dv_create_date", DataType::Utf8View, true),
    Field::new("dv_create_time", DataType::Utf8View, true),
    Field::new("dv_cmdline_args", DataType::Utf8View, true),
]))
}
