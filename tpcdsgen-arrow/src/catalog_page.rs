use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogPageRowGenerator, GeneratedRow, RowGenerator};

pub struct CatalogPageArrow {
    generator: CatalogPageRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl CatalogPageArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogPage);
        Self { generator: CatalogPageRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for CatalogPageArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for CatalogPageArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut cp_sk: Vec<Option<i64>> = Vec::new();
        let mut cp_id: Vec<Option<String>> = Vec::new();
        let mut cp_start: Vec<Option<i64>> = Vec::new();
        let mut cp_end: Vec<Option<i64>> = Vec::new();
        let mut cp_dept: Vec<Option<String>> = Vec::new();
        let mut cp_num: Vec<Option<i32>> = Vec::new();
        let mut cp_page_num: Vec<Option<i32>> = Vec::new();
        let mut cp_desc: Vec<Option<String>> = Vec::new();
        let mut cp_type: Vec<Option<String>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::CatalogPage(r) = g {
                    let nbm = r.null_bit_map();
                    cp_sk.push(sk_opt(nbm, 0, r.get_cp_catalog_page_sk()));
                    cp_id.push(opt(nbm, 1, r.get_cp_catalog_page_id().to_owned()));
                    cp_start.push(sk_opt(nbm, 2, r.get_cp_start_date_id()));
                    cp_end.push(sk_opt(nbm, 3, r.get_cp_end_date_id()));
                    cp_dept.push(opt(nbm, 4, r.get_cp_department().to_owned()));
                    cp_num.push(opt(nbm, 5, r.get_cp_catalog_number()));
                    cp_page_num.push(opt(nbm, 6, r.get_cp_catalog_page_number()));
                    cp_desc.push(opt(nbm, 7, r.get_cp_description().to_owned()));
                    cp_type.push(opt(nbm, 8, r.get_cp_type().to_owned()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if cp_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(cp_sk)),
            Arc::new(string_view_array_from_opt_iter(cp_id.iter().map(|s| s.as_deref()))),
            Arc::new(Int64Array::from(cp_start)),
            Arc::new(Int64Array::from(cp_end)),
            Arc::new(string_view_array_from_opt_iter(cp_dept.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(cp_num)),
            Arc::new(Int32Array::from(cp_page_num)),
            Arc::new(string_view_array_from_opt_iter(cp_desc.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(cp_type.iter().map(|s| s.as_deref()))),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("cp_catalog_page_sk", DataType::Int64, true),
    Field::new("cp_catalog_page_id", DataType::Utf8View, true),
    Field::new("cp_start_date_sk", DataType::Int64, true),
    Field::new("cp_end_date_sk", DataType::Int64, true),
    Field::new("cp_department", DataType::Utf8View, true),
    Field::new("cp_catalog_number", DataType::Int32, true),
    Field::new("cp_catalog_page_number", DataType::Int32, true),
    Field::new("cp_description", DataType::Utf8View, true),
    Field::new("cp_type", DataType::Utf8View, true),
])));
