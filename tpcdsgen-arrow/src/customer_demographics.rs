use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerDemographicsRowGenerator, GeneratedRow, RowGenerator};

pub struct CustomerDemographicsArrow {
    generator: CustomerDemographicsRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl CustomerDemographicsArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CustomerDemographics);
        Self { generator: CustomerDemographicsRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for CustomerDemographicsArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for CustomerDemographicsArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut demo_sk: Vec<Option<i64>> = Vec::new();
        let mut gender: Vec<Option<String>> = Vec::new();
        let mut marital: Vec<Option<String>> = Vec::new();
        let mut education: Vec<Option<String>> = Vec::new();
        let mut purchase: Vec<Option<i32>> = Vec::new();
        let mut credit: Vec<Option<String>> = Vec::new();
        let mut dep_count: Vec<Option<i32>> = Vec::new();
        let mut dep_emp: Vec<Option<i32>> = Vec::new();
        let mut dep_college: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::CustomerDemographics(r) = g {
                    let nbm = r.null_bit_map();
                    demo_sk.push(sk_opt(nbm, 0, r.get_cd_demo_sk()));
                    gender.push(opt(nbm, 1, r.get_cd_gender().to_owned()));
                    marital.push(opt(nbm, 2, r.get_cd_marital_status().to_owned()));
                    education.push(opt(nbm, 3, r.get_cd_education_status().to_owned()));
                    purchase.push(opt(nbm, 4, r.get_cd_purchase_estimate()));
                    credit.push(opt(nbm, 5, r.get_cd_credit_rating().to_owned()));
                    dep_count.push(opt(nbm, 6, r.get_cd_dep_count()));
                    dep_emp.push(opt(nbm, 7, r.get_cd_dep_employed_count()));
                    dep_college.push(opt(nbm, 8, r.get_cd_dep_college_count()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if demo_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(demo_sk)),
            Arc::new(string_view_array_from_opt_iter(gender.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(marital.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(education.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(purchase)),
            Arc::new(string_view_array_from_opt_iter(credit.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(dep_count)),
            Arc::new(Int32Array::from(dep_emp)),
            Arc::new(Int32Array::from(dep_college)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("cd_demo_sk", DataType::Int64, false),
    Field::new("cd_gender", DataType::Utf8View, false),
    Field::new("cd_marital_status", DataType::Utf8View, false),
    Field::new("cd_education_status", DataType::Utf8View, false),
    Field::new("cd_purchase_estimate", DataType::Int32, false),
    Field::new("cd_credit_rating", DataType::Utf8View, false),
    Field::new("cd_dep_count", DataType::Int32, false),
    Field::new("cd_dep_employed_count", DataType::Int32, false),
    Field::new("cd_dep_college_count", DataType::Int32, false),
])));
