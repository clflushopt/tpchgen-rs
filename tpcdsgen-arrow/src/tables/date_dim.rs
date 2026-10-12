use crate::conversions::{
    bool_to_yn, date_to_date32, PrimitiveArrayRecycler, StringViewArrayRecycler,
};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DateDimRow, DateDimRowGenerator};

/// Generate [`DateDimRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct DateDimArrow {
    inner: DateDimRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<DateDimRow>,
    d_date_sk: PrimitiveArrayRecycler<Int32Type>,
    d_date_id: StringViewArrayRecycler,
    d_date: PrimitiveArrayRecycler<Date32Type>,
    d_month_seq: PrimitiveArrayRecycler<Int32Type>,
    d_week_seq: PrimitiveArrayRecycler<Int32Type>,
    d_quarter_seq: PrimitiveArrayRecycler<Int32Type>,
    d_year: PrimitiveArrayRecycler<Int32Type>,
    d_dow: PrimitiveArrayRecycler<Int32Type>,
    d_moy: PrimitiveArrayRecycler<Int32Type>,
    d_dom: PrimitiveArrayRecycler<Int32Type>,
    d_qoy: PrimitiveArrayRecycler<Int32Type>,
    d_fy_year: PrimitiveArrayRecycler<Int32Type>,
    d_fy_quarter_seq: PrimitiveArrayRecycler<Int32Type>,
    d_fy_week_seq: PrimitiveArrayRecycler<Int32Type>,
    d_day_name: StringViewArrayRecycler,
    d_quarter_name: StringViewArrayRecycler,
    d_holiday: StringViewArrayRecycler,
    d_weekend: StringViewArrayRecycler,
    d_following_holiday: StringViewArrayRecycler,
    d_first_dom: PrimitiveArrayRecycler<Int32Type>,
    d_last_dom: PrimitiveArrayRecycler<Int32Type>,
    d_same_day_ly: PrimitiveArrayRecycler<Int32Type>,
    d_same_day_lq: PrimitiveArrayRecycler<Int32Type>,
    d_current_day: StringViewArrayRecycler,
    d_current_week: StringViewArrayRecycler,
    d_current_month: StringViewArrayRecycler,
    d_current_quarter: StringViewArrayRecycler,
    d_current_year: StringViewArrayRecycler,
}

impl DateDimArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DateDim);
        Self {
            inner: DateDimRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            d_date_sk: PrimitiveArrayRecycler::new(),
            d_date_id: StringViewArrayRecycler::new(),
            d_date: PrimitiveArrayRecycler::new(),
            d_month_seq: PrimitiveArrayRecycler::new(),
            d_week_seq: PrimitiveArrayRecycler::new(),
            d_quarter_seq: PrimitiveArrayRecycler::new(),
            d_year: PrimitiveArrayRecycler::new(),
            d_dow: PrimitiveArrayRecycler::new(),
            d_moy: PrimitiveArrayRecycler::new(),
            d_dom: PrimitiveArrayRecycler::new(),
            d_qoy: PrimitiveArrayRecycler::new(),
            d_fy_year: PrimitiveArrayRecycler::new(),
            d_fy_quarter_seq: PrimitiveArrayRecycler::new(),
            d_fy_week_seq: PrimitiveArrayRecycler::new(),
            d_day_name: StringViewArrayRecycler::new(),
            d_quarter_name: StringViewArrayRecycler::new(),
            d_holiday: StringViewArrayRecycler::new(),
            d_weekend: StringViewArrayRecycler::new(),
            d_following_holiday: StringViewArrayRecycler::new(),
            d_first_dom: PrimitiveArrayRecycler::new(),
            d_last_dom: PrimitiveArrayRecycler::new(),
            d_same_day_ly: PrimitiveArrayRecycler::new(),
            d_same_day_lq: PrimitiveArrayRecycler::new(),
            d_current_day: StringViewArrayRecycler::new(),
            d_current_week: StringViewArrayRecycler::new(),
            d_current_month: StringViewArrayRecycler::new(),
            d_current_quarter: StringViewArrayRecycler::new(),
            d_current_year: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for DateDimArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for DateDimArrow {
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
            d_date_sk,
            d_date_id,
            d_date,
            d_month_seq,
            d_week_seq,
            d_quarter_seq,
            d_year,
            d_dow,
            d_moy,
            d_dom,
            d_qoy,
            d_fy_year,
            d_fy_quarter_seq,
            d_fy_week_seq,
            d_day_name,
            d_quarter_name,
            d_holiday,
            d_weekend,
            d_following_holiday,
            d_first_dom,
            d_last_dom,
            d_same_day_ly,
            d_same_day_lq,
            d_current_day,
            d_current_week,
            d_current_month,
            d_current_quarter,
            d_current_year,
            ..
        } = self;
        let nbm = DateDimRow::null_bit_map;
        // Only d_date_sk consults the null bitmap; every other column is non-null.
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, f: fn(&DateDimRow) -> i32| {
            rec.build(rows, f, |_| true)
        };
        let str = |rec: &mut StringViewArrayRecycler, f: fn(&DateDimRow) -> &str| {
            rec.build(rows, |r| Some(f(r)))
        };

        let d_date_sk = d_date_sk.int32_sk(rows, 0, nbm, |r| r.d_date_sk);
        let d_date_id = str(d_date_id, |r| r.d_date_id.as_str());
        let d_date = d_date.build(rows, |r| date_to_date32(&r.d_date), |_| true);
        let d_month_seq = int(d_month_seq, |r| r.d_month_seq);
        let d_week_seq = int(d_week_seq, |r| r.d_week_seq);
        let d_quarter_seq = int(d_quarter_seq, |r| r.d_quarter_seq);
        let d_year = int(d_year, |r| r.d_year);
        let d_dow = int(d_dow, |r| r.d_dow);
        let d_moy = int(d_moy, |r| r.d_moy);
        let d_dom = int(d_dom, |r| r.d_dom);
        let d_qoy = int(d_qoy, |r| r.d_qoy);
        let d_fy_year = int(d_fy_year, |r| r.d_fy_year);
        let d_fy_quarter_seq = int(d_fy_quarter_seq, |r| r.d_fy_quarter_seq);
        let d_fy_week_seq = int(d_fy_week_seq, |r| r.d_fy_week_seq);
        let d_day_name = str(d_day_name, |r| r.d_day_name.as_str());
        let d_quarter_name = str(d_quarter_name, |r| r.d_quarter_name.as_str());
        let d_holiday = str(d_holiday, |r| bool_to_yn(r.d_holiday));
        let d_weekend = str(d_weekend, |r| bool_to_yn(r.d_weekend));
        let d_following_holiday = str(d_following_holiday, |r| bool_to_yn(r.d_following_holiday));
        let d_first_dom = int(d_first_dom, |r| r.d_first_dom);
        let d_last_dom = int(d_last_dom, |r| r.d_last_dom);
        let d_same_day_ly = int(d_same_day_ly, |r| r.d_same_day_ly);
        let d_same_day_lq = int(d_same_day_lq, |r| r.d_same_day_lq);
        let d_current_day = str(d_current_day, |r| bool_to_yn(r.d_current_day));
        let d_current_week = str(d_current_week, |r| bool_to_yn(r.d_current_week));
        let d_current_month = str(d_current_month, |r| bool_to_yn(r.d_current_month));
        let d_current_quarter = str(d_current_quarter, |r| bool_to_yn(r.d_current_quarter));
        let d_current_year = str(d_current_year, |r| bool_to_yn(r.d_current_year));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(d_date_sk),
                Arc::new(d_date_id),
                Arc::new(d_date),
                Arc::new(d_month_seq),
                Arc::new(d_week_seq),
                Arc::new(d_quarter_seq),
                Arc::new(d_year),
                Arc::new(d_dow),
                Arc::new(d_moy),
                Arc::new(d_dom),
                Arc::new(d_qoy),
                Arc::new(d_fy_year),
                Arc::new(d_fy_quarter_seq),
                Arc::new(d_fy_week_seq),
                Arc::new(d_day_name),
                Arc::new(d_quarter_name),
                Arc::new(d_holiday),
                Arc::new(d_weekend),
                Arc::new(d_following_holiday),
                Arc::new(d_first_dom),
                Arc::new(d_last_dom),
                Arc::new(d_same_day_ly),
                Arc::new(d_same_day_lq),
                Arc::new(d_current_day),
                Arc::new(d_current_week),
                Arc::new(d_current_month),
                Arc::new(d_current_quarter),
                Arc::new(d_current_year),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("d_date_sk", DataType::Int32, false),
        Field::new("d_date_id", DataType::Utf8View, false),
        Field::new("d_date", DataType::Date32, true),
        Field::new("d_month_seq", DataType::Int32, true),
        Field::new("d_week_seq", DataType::Int32, true),
        Field::new("d_quarter_seq", DataType::Int32, true),
        Field::new("d_year", DataType::Int32, true),
        Field::new("d_dow", DataType::Int32, true),
        Field::new("d_moy", DataType::Int32, true),
        Field::new("d_dom", DataType::Int32, true),
        Field::new("d_qoy", DataType::Int32, true),
        Field::new("d_fy_year", DataType::Int32, true),
        Field::new("d_fy_quarter_seq", DataType::Int32, true),
        Field::new("d_fy_week_seq", DataType::Int32, true),
        Field::new("d_day_name", DataType::Utf8View, true),
        Field::new("d_quarter_name", DataType::Utf8View, true),
        Field::new("d_holiday", DataType::Utf8View, true),
        Field::new("d_weekend", DataType::Utf8View, true),
        Field::new("d_following_holiday", DataType::Utf8View, true),
        Field::new("d_first_dom", DataType::Int32, true),
        Field::new("d_last_dom", DataType::Int32, true),
        Field::new("d_same_day_ly", DataType::Int32, true),
        Field::new("d_same_day_lq", DataType::Int32, true),
        Field::new("d_current_day", DataType::Utf8View, true),
        Field::new("d_current_week", DataType::Utf8View, true),
        Field::new("d_current_month", DataType::Utf8View, true),
        Field::new("d_current_quarter", DataType::Utf8View, true),
        Field::new("d_current_year", DataType::Utf8View, true),
    ]))
}
