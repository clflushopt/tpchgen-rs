use crate::conversions::{bool_to_yn, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{WebPageRow, WebPageRowGenerator};

/// Generate [`WebPageRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct WebPageArrow {
    inner: WebPageRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<WebPageRow>,
    wp_web_page_sk: PrimitiveArrayRecycler<Int32Type>,
    wp_web_page_id: StringViewArrayRecycler,
    wp_rec_start_date: PrimitiveArrayRecycler<Date32Type>,
    wp_rec_end_date: PrimitiveArrayRecycler<Date32Type>,
    wp_creation_date_sk: PrimitiveArrayRecycler<Int32Type>,
    wp_access_date_sk: PrimitiveArrayRecycler<Int32Type>,
    wp_autogen_flag: StringViewArrayRecycler,
    wp_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    wp_url: StringViewArrayRecycler,
    wp_type: StringViewArrayRecycler,
    wp_char_count: PrimitiveArrayRecycler<Int32Type>,
    wp_link_count: PrimitiveArrayRecycler<Int32Type>,
    wp_image_count: PrimitiveArrayRecycler<Int32Type>,
    wp_max_ad_count: PrimitiveArrayRecycler<Int32Type>,
}

impl WebPageArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebPage);
        Self {
            inner: WebPageRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            wp_web_page_sk: PrimitiveArrayRecycler::new(),
            wp_web_page_id: StringViewArrayRecycler::new(),
            wp_rec_start_date: PrimitiveArrayRecycler::new(),
            wp_rec_end_date: PrimitiveArrayRecycler::new(),
            wp_creation_date_sk: PrimitiveArrayRecycler::new(),
            wp_access_date_sk: PrimitiveArrayRecycler::new(),
            wp_autogen_flag: StringViewArrayRecycler::new(),
            wp_customer_sk: PrimitiveArrayRecycler::new(),
            wp_url: StringViewArrayRecycler::new(),
            wp_type: StringViewArrayRecycler::new(),
            wp_char_count: PrimitiveArrayRecycler::new(),
            wp_link_count: PrimitiveArrayRecycler::new(),
            wp_image_count: PrimitiveArrayRecycler::new(),
            wp_max_ad_count: PrimitiveArrayRecycler::new(),
        }
    }
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.inner
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate only source rows `starting_row_number..=ending_row_number`
    /// (1-based, inclusive). The ending row number is clamped to the table's
    /// row count.
    pub fn with_source_row_range(
        mut self,
        starting_row_number: u64,
        ending_row_number: u64,
    ) -> Self {
        self.inner
            .set_source_row_range(starting_row_number, ending_row_number);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchReader for WebPageArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WebPageArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scratch.clear();
        self.scratch
            .extend(self.inner.by_ref().take(self.batch_size));
        let rows = &self.scratch;
        if rows.is_empty() {
            return None;
        }

        let schema = self.schema();
        let Self {
            wp_web_page_sk,
            wp_web_page_id,
            wp_rec_start_date,
            wp_rec_end_date,
            wp_creation_date_sk,
            wp_access_date_sk,
            wp_autogen_flag,
            wp_customer_sk,
            wp_url,
            wp_type,
            wp_char_count,
            wp_link_count,
            wp_image_count,
            wp_max_ad_count,
            ..
        } = self;
        let nbm = WebPageRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebPageRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebPageRow) -> i32| {
            rec.int32(rows, pos, nbm, f)
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&WebPageRow) -> &str| {
            rec.string(rows, pos, nbm, f)
        };

        let wp_web_page_sk = sk32(wp_web_page_sk, 0, WebPageRow::get_wp_page_sk);
        let wp_web_page_id = str(wp_web_page_id, 1, WebPageRow::get_wp_page_id);
        let wp_rec_start_date =
            wp_rec_start_date.date32_julian(rows, 2, nbm, WebPageRow::get_wp_rec_start_date_id);
        // wp_rec_end_date ignores the null bitmap; only the negative-Julian sentinel is NULL
        let wp_rec_end_date =
            wp_rec_end_date.date32_julian(rows, 3, |_| 0, WebPageRow::get_wp_rec_end_date_id);
        let wp_creation_date_sk = sk32(wp_creation_date_sk, 4, WebPageRow::get_wp_creation_date_sk);
        let wp_access_date_sk = sk32(wp_access_date_sk, 5, WebPageRow::get_wp_access_date_sk);
        let wp_autogen_flag = str(wp_autogen_flag, 6, |r| bool_to_yn(r.get_wp_autogen_flag()));
        let wp_customer_sk = sk32(wp_customer_sk, 7, WebPageRow::get_wp_customer_sk);
        let wp_url = str(wp_url, 8, WebPageRow::get_wp_url);
        let wp_type = str(wp_type, 9, WebPageRow::get_wp_type);
        let wp_char_count = int(wp_char_count, 10, WebPageRow::get_wp_char_count);
        let wp_link_count = int(wp_link_count, 11, WebPageRow::get_wp_link_count);
        let wp_image_count = int(wp_image_count, 12, WebPageRow::get_wp_image_count);
        let wp_max_ad_count = int(wp_max_ad_count, 13, WebPageRow::get_wp_max_ad_count);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(wp_web_page_sk),
                Arc::new(wp_web_page_id),
                Arc::new(wp_rec_start_date),
                Arc::new(wp_rec_end_date),
                Arc::new(wp_creation_date_sk),
                Arc::new(wp_access_date_sk),
                Arc::new(wp_autogen_flag),
                Arc::new(wp_customer_sk),
                Arc::new(wp_url),
                Arc::new(wp_type),
                Arc::new(wp_char_count),
                Arc::new(wp_link_count),
                Arc::new(wp_image_count),
                Arc::new(wp_max_ad_count),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("wp_web_page_sk", DataType::Int32, false),
        Field::new("wp_web_page_id", DataType::Utf8View, false),
        Field::new("wp_rec_start_date", DataType::Date32, true),
        Field::new("wp_rec_end_date", DataType::Date32, true),
        Field::new("wp_creation_date_sk", DataType::Int32, true),
        Field::new("wp_access_date_sk", DataType::Int32, true),
        Field::new("wp_autogen_flag", DataType::Utf8View, true),
        Field::new("wp_customer_sk", DataType::Int32, true),
        Field::new("wp_url", DataType::Utf8View, true),
        Field::new("wp_type", DataType::Utf8View, true),
        Field::new("wp_char_count", DataType::Int32, true),
        Field::new("wp_link_count", DataType::Int32, true),
        Field::new("wp_image_count", DataType::Int32, true),
        Field::new("wp_max_ad_count", DataType::Int32, true),
    ]))
}
