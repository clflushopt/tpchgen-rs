use crate::conversions::{bool_to_yn, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{PromotionRow, PromotionRowGenerator};

/// Generate [`PromotionRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct PromotionArrow {
    inner: PromotionRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<PromotionRow>,
    p_promo_sk: PrimitiveArrayRecycler<Int32Type>,
    p_promo_id: StringViewArrayRecycler,
    p_start_date_sk: PrimitiveArrayRecycler<Int32Type>,
    p_end_date_sk: PrimitiveArrayRecycler<Int32Type>,
    p_item_sk: PrimitiveArrayRecycler<Int32Type>,
    p_cost: PrimitiveArrayRecycler<Decimal128Type>,
    p_response_target: PrimitiveArrayRecycler<Int32Type>,
    p_promo_name: StringViewArrayRecycler,
    p_channel_dmail: StringViewArrayRecycler,
    p_channel_email: StringViewArrayRecycler,
    p_channel_catalog: StringViewArrayRecycler,
    p_channel_tv: StringViewArrayRecycler,
    p_channel_radio: StringViewArrayRecycler,
    p_channel_press: StringViewArrayRecycler,
    p_channel_event: StringViewArrayRecycler,
    p_channel_demo: StringViewArrayRecycler,
    p_channel_details: StringViewArrayRecycler,
    p_purpose: StringViewArrayRecycler,
    p_discount_active: StringViewArrayRecycler,
}

impl PromotionArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Promotion);
        Self {
            inner: PromotionRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            p_promo_sk: PrimitiveArrayRecycler::new(),
            p_promo_id: StringViewArrayRecycler::new(),
            p_start_date_sk: PrimitiveArrayRecycler::new(),
            p_end_date_sk: PrimitiveArrayRecycler::new(),
            p_item_sk: PrimitiveArrayRecycler::new(),
            p_cost: PrimitiveArrayRecycler::new(),
            p_response_target: PrimitiveArrayRecycler::new(),
            p_promo_name: StringViewArrayRecycler::new(),
            p_channel_dmail: StringViewArrayRecycler::new(),
            p_channel_email: StringViewArrayRecycler::new(),
            p_channel_catalog: StringViewArrayRecycler::new(),
            p_channel_tv: StringViewArrayRecycler::new(),
            p_channel_radio: StringViewArrayRecycler::new(),
            p_channel_press: StringViewArrayRecycler::new(),
            p_channel_event: StringViewArrayRecycler::new(),
            p_channel_demo: StringViewArrayRecycler::new(),
            p_channel_details: StringViewArrayRecycler::new(),
            p_purpose: StringViewArrayRecycler::new(),
            p_discount_active: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for PromotionArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for PromotionArrow {
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
            p_promo_sk,
            p_promo_id,
            p_start_date_sk,
            p_end_date_sk,
            p_item_sk,
            p_cost,
            p_response_target,
            p_promo_name,
            p_channel_dmail,
            p_channel_email,
            p_channel_catalog,
            p_channel_tv,
            p_channel_radio,
            p_channel_press,
            p_channel_event,
            p_channel_demo,
            p_channel_details,
            p_purpose,
            p_discount_active,
            ..
        } = self;
        let nbm = PromotionRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&PromotionRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>,
                   pos,
                   f: fn(&PromotionRow) -> i32| { rec.int32(rows, pos, nbm, f) };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&PromotionRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let p_promo_sk = sk32(p_promo_sk, 0, PromotionRow::get_p_promo_sk);
        let p_promo_id = str(p_promo_id, 1, PromotionRow::get_p_promo_id);
        let p_start_date_sk = sk32(p_start_date_sk, 2, PromotionRow::get_p_start_date_id);
        let p_end_date_sk = sk32(p_end_date_sk, 3, PromotionRow::get_p_end_date_id);
        let p_item_sk = sk32(p_item_sk, 4, PromotionRow::get_p_item_sk);
        let p_cost = p_cost.decimal_15_2(rows, 5, nbm, PromotionRow::get_p_cost);
        let p_response_target = int(p_response_target, 6, PromotionRow::get_p_response_target);
        let p_promo_name = str(p_promo_name, 7, PromotionRow::get_p_promo_name);
        let p_channel_dmail = str(p_channel_dmail, 8, |r| bool_to_yn(r.get_p_channel_dmail()));
        let p_channel_email = str(p_channel_email, 9, |r| bool_to_yn(r.get_p_channel_email()));
        let p_channel_catalog = str(p_channel_catalog, 10, |r| {
            bool_to_yn(r.get_p_channel_catalog())
        });
        let p_channel_tv = str(p_channel_tv, 11, |r| bool_to_yn(r.get_p_channel_tv()));
        let p_channel_radio = str(p_channel_radio, 12, |r| bool_to_yn(r.get_p_channel_radio()));
        let p_channel_press = str(p_channel_press, 13, |r| bool_to_yn(r.get_p_channel_press()));
        let p_channel_event = str(p_channel_event, 14, |r| bool_to_yn(r.get_p_channel_event()));
        let p_channel_demo = str(p_channel_demo, 15, |r| bool_to_yn(r.get_p_channel_demo()));
        let p_channel_details = str(p_channel_details, 16, PromotionRow::get_p_channel_details);
        let p_purpose = str(p_purpose, 17, PromotionRow::get_p_purpose);
        let p_discount_active = str(p_discount_active, 18, |r| {
            bool_to_yn(r.get_p_discount_active())
        });

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(p_promo_sk),
                Arc::new(p_promo_id),
                Arc::new(p_start_date_sk),
                Arc::new(p_end_date_sk),
                Arc::new(p_item_sk),
                Arc::new(p_cost),
                Arc::new(p_response_target),
                Arc::new(p_promo_name),
                Arc::new(p_channel_dmail),
                Arc::new(p_channel_email),
                Arc::new(p_channel_catalog),
                Arc::new(p_channel_tv),
                Arc::new(p_channel_radio),
                Arc::new(p_channel_press),
                Arc::new(p_channel_event),
                Arc::new(p_channel_demo),
                Arc::new(p_channel_details),
                Arc::new(p_purpose),
                Arc::new(p_discount_active),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("p_promo_sk", DataType::Int32, false),
        Field::new("p_promo_id", DataType::Utf8View, false),
        Field::new("p_start_date_sk", DataType::Int32, true),
        Field::new("p_end_date_sk", DataType::Int32, true),
        Field::new("p_item_sk", DataType::Int32, true),
        Field::new("p_cost", DataType::Decimal128(15, 2), true),
        Field::new("p_response_target", DataType::Int32, true),
        Field::new("p_promo_name", DataType::Utf8View, true),
        Field::new("p_channel_dmail", DataType::Utf8View, true),
        Field::new("p_channel_email", DataType::Utf8View, true),
        Field::new("p_channel_catalog", DataType::Utf8View, true),
        Field::new("p_channel_tv", DataType::Utf8View, true),
        Field::new("p_channel_radio", DataType::Utf8View, true),
        Field::new("p_channel_press", DataType::Utf8View, true),
        Field::new("p_channel_event", DataType::Utf8View, true),
        Field::new("p_channel_demo", DataType::Utf8View, true),
        Field::new("p_channel_details", DataType::Utf8View, true),
        Field::new("p_purpose", DataType::Utf8View, true),
        Field::new("p_discount_active", DataType::Utf8View, true),
    ]))
}
