use crate::conversions::{bool_to_yn, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerRowGenerator, GeneratedRow, RowGenerator};

pub struct CustomerArrow {
    generator: CustomerRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl CustomerArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Customer);
        Self { generator: CustomerRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for CustomerArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for CustomerArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut c_sk: Vec<Option<i64>> = Vec::new();
        let mut c_id: Vec<Option<String>> = Vec::new();
        let mut c_cdemo_sk: Vec<Option<i64>> = Vec::new();
        let mut c_hdemo_sk: Vec<Option<i64>> = Vec::new();
        let mut c_addr_sk: Vec<Option<i64>> = Vec::new();
        let mut c_shipto_date: Vec<Option<i32>> = Vec::new();
        let mut c_sales_date: Vec<Option<i32>> = Vec::new();
        let mut c_salutation: Vec<Option<String>> = Vec::new();
        let mut c_first_name: Vec<Option<String>> = Vec::new();
        let mut c_last_name: Vec<Option<String>> = Vec::new();
        let mut c_pref_flag: Vec<Option<String>> = Vec::new();
        let mut c_birth_day: Vec<Option<i32>> = Vec::new();
        let mut c_birth_month: Vec<Option<i32>> = Vec::new();
        let mut c_birth_year: Vec<Option<i32>> = Vec::new();
        let mut c_birth_country: Vec<Option<String>> = Vec::new();
        let mut c_login: Vec<Option<String>> = Vec::new(); // always null
        let mut c_email: Vec<Option<String>> = Vec::new();
        let mut c_last_review: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::Customer(r) = g {
                    let nbm = r.null_bit_map();
                    c_sk.push(sk_opt(nbm, 0, r.get_c_customer_sk()));
                    c_id.push(opt(nbm, 1, r.get_c_customer_id().to_owned()));
                    c_cdemo_sk.push(sk_opt(nbm, 2, r.get_c_current_cdemo_sk()));
                    c_hdemo_sk.push(sk_opt(nbm, 3, r.get_c_current_hdemo_sk()));
                    c_addr_sk.push(sk_opt(nbm, 4, r.get_c_current_addr_sk()));
                    c_shipto_date.push(opt(nbm, 5, r.get_c_first_shipto_date_id()));
                    c_sales_date.push(opt(nbm, 6, r.get_c_first_sales_date_id()));
                    c_salutation.push(opt(nbm, 7, r.get_c_salutation().to_owned()));
                    c_first_name.push(opt(nbm, 8, r.get_c_first_name().to_owned()));
                    c_last_name.push(opt(nbm, 9, r.get_c_last_name().to_owned()));
                    c_pref_flag.push(opt(nbm, 10, bool_to_yn(r.get_c_preferred_cust_flag()).to_owned()));
                    c_birth_day.push(opt(nbm, 11, r.get_c_birth_day()));
                    c_birth_month.push(opt(nbm, 12, r.get_c_birth_month()));
                    c_birth_year.push(opt(nbm, 13, r.get_c_birth_year()));
                    c_birth_country.push(opt(nbm, 14, r.get_c_birth_country().to_owned()));
                    c_login.push(None); // always null per TPC-DS spec
                    c_email.push(opt(nbm, 16, r.get_c_email_address().to_owned()));
                    c_last_review.push(opt(nbm, 17, r.get_c_last_review_date()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if c_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(c_sk)),
            Arc::new(string_view_array_from_opt_iter(c_id.iter().map(|s| s.as_deref()))),
            Arc::new(Int64Array::from(c_cdemo_sk)),
            Arc::new(Int64Array::from(c_hdemo_sk)),
            Arc::new(Int64Array::from(c_addr_sk)),
            Arc::new(Int32Array::from(c_shipto_date)),
            Arc::new(Int32Array::from(c_sales_date)),
            Arc::new(string_view_array_from_opt_iter(c_salutation.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(c_first_name.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(c_last_name.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(c_pref_flag.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(c_birth_day)),
            Arc::new(Int32Array::from(c_birth_month)),
            Arc::new(Int32Array::from(c_birth_year)),
            Arc::new(string_view_array_from_opt_iter(c_birth_country.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(c_login.iter().map(|_| None::<&str>))),
            Arc::new(string_view_array_from_opt_iter(c_email.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(c_last_review)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("c_customer_sk", DataType::Int64, true),
    Field::new("c_customer_id", DataType::Utf8View, true),
    Field::new("c_current_cdemo_sk", DataType::Int64, true),
    Field::new("c_current_hdemo_sk", DataType::Int64, true),
    Field::new("c_current_addr_sk", DataType::Int64, true),
    Field::new("c_first_shipto_date_sk", DataType::Int32, true),
    Field::new("c_first_sales_date_sk", DataType::Int32, true),
    Field::new("c_salutation", DataType::Utf8View, true),
    Field::new("c_first_name", DataType::Utf8View, true),
    Field::new("c_last_name", DataType::Utf8View, true),
    Field::new("c_preferred_cust_flag", DataType::Utf8View, true),
    Field::new("c_birth_day", DataType::Int32, true),
    Field::new("c_birth_month", DataType::Int32, true),
    Field::new("c_birth_year", DataType::Int32, true),
    Field::new("c_birth_country", DataType::Utf8View, true),
    Field::new("c_login", DataType::Utf8View, true),
    Field::new("c_email_address", DataType::Utf8View, true),
    Field::new("c_last_review_date_sk", DataType::Int32, true),
])));
