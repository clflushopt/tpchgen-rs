use crate::conversions::{bool_to_yn, julian_to_date32, opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Date32Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, WebPageRowGenerator};

pub struct WebPageArrow {
    generator: WebPageRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl WebPageArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebPage);
        Self { generator: WebPageRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for WebPageArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for WebPageArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut wp_sk: Vec<Option<i64>> = Vec::new();
        let mut wp_id: Vec<Option<String>> = Vec::new();
        let mut wp_rec_start: Vec<Option<i32>> = Vec::new();
        let mut wp_rec_end: Vec<Option<i32>> = Vec::new();
        let mut wp_creation_date: Vec<Option<i64>> = Vec::new();
        let mut wp_access_date: Vec<Option<i64>> = Vec::new();
        let mut wp_autogen: Vec<Option<&'static str>> = Vec::new();
        let mut wp_customer: Vec<Option<i64>> = Vec::new();
        let mut wp_url: Vec<Option<String>> = Vec::new();
        let mut wp_type: Vec<Option<String>> = Vec::new();
        let mut wp_char_count: Vec<Option<i32>> = Vec::new();
        let mut wp_link_count: Vec<Option<i32>> = Vec::new();
        let mut wp_image_count: Vec<Option<i32>> = Vec::new();
        let mut wp_max_ad_count: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::WebPage(r) = g {
                    let nbm = r.null_bit_map();
                    wp_sk.push(sk_opt(nbm, 0, r.get_wp_page_sk()));
                    wp_id.push(opt(nbm, 1, r.get_wp_page_id().to_owned()));
                    wp_rec_start.push(julian_to_date32(r.get_wp_rec_start_date_id()));
                    wp_rec_end.push(julian_to_date32(r.get_wp_rec_end_date_id()));
                    wp_creation_date.push(sk_opt(nbm, 4, r.get_wp_creation_date_sk()));
                    wp_access_date.push(sk_opt(nbm, 5, r.get_wp_access_date_sk()));
                    wp_autogen.push(opt(nbm, 6, bool_to_yn(r.get_wp_autogen_flag())));
                    wp_customer.push(sk_opt(nbm, 7, r.get_wp_customer_sk()));
                    wp_url.push(opt(nbm, 8, r.get_wp_url().to_owned()));
                    wp_type.push(opt(nbm, 9, r.get_wp_type().to_owned()));
                    wp_char_count.push(opt(nbm, 10, r.get_wp_char_count()));
                    wp_link_count.push(opt(nbm, 11, r.get_wp_link_count()));
                    wp_image_count.push(opt(nbm, 12, r.get_wp_image_count()));
                    wp_max_ad_count.push(opt(nbm, 13, r.get_wp_max_ad_count()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if wp_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(wp_sk)),
            Arc::new(string_view_array_from_opt_iter(wp_id.iter().map(|s| s.as_deref()))),
            Arc::new(Date32Array::from(wp_rec_start)),
            Arc::new(Date32Array::from(wp_rec_end)),
            Arc::new(Int64Array::from(wp_creation_date)),
            Arc::new(Int64Array::from(wp_access_date)),
            Arc::new(string_view_array_from_opt_iter(wp_autogen.iter().copied())),
            Arc::new(Int64Array::from(wp_customer)),
            Arc::new(string_view_array_from_opt_iter(wp_url.iter().map(|s| s.as_deref()))),
            Arc::new(string_view_array_from_opt_iter(wp_type.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(wp_char_count)),
            Arc::new(Int32Array::from(wp_link_count)),
            Arc::new(Int32Array::from(wp_image_count)),
            Arc::new(Int32Array::from(wp_max_ad_count)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("wp_web_page_sk", DataType::Int64, true),
    Field::new("wp_web_page_id", DataType::Utf8View, true),
    Field::new("wp_rec_start_date", DataType::Date32, true),
    Field::new("wp_rec_end_date", DataType::Date32, true),
    Field::new("wp_creation_date_sk", DataType::Int64, true),
    Field::new("wp_access_date_sk", DataType::Int64, true),
    Field::new("wp_autogen_flag", DataType::Utf8View, true),
    Field::new("wp_customer_sk", DataType::Int64, true),
    Field::new("wp_url", DataType::Utf8View, true),
    Field::new("wp_type", DataType::Utf8View, true),
    Field::new("wp_char_count", DataType::Int32, true),
    Field::new("wp_link_count", DataType::Int32, true),
    Field::new("wp_image_count", DataType::Int32, true),
    Field::new("wp_max_ad_count", DataType::Int32, true),
])));
