use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{TimeDimRow, TimeDimRowGenerator};

/// Generate [`TimeDimRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct TimeDimArrow {
    inner: TimeDimRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<TimeDimRow>,
    t_time_sk: PrimitiveArrayRecycler<Int32Type>,
    t_time_id: StringViewArrayRecycler,
    t_time: PrimitiveArrayRecycler<Int32Type>,
    t_hour: PrimitiveArrayRecycler<Int32Type>,
    t_minute: PrimitiveArrayRecycler<Int32Type>,
    t_second: PrimitiveArrayRecycler<Int32Type>,
    t_am_pm: StringViewArrayRecycler,
    t_shift: StringViewArrayRecycler,
    t_sub_shift: StringViewArrayRecycler,
    t_meal_time: StringViewArrayRecycler,
}

impl TimeDimArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::TimeDim);
        Self {
            inner: TimeDimRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            t_time_sk: PrimitiveArrayRecycler::new(),
            t_time_id: StringViewArrayRecycler::new(),
            t_time: PrimitiveArrayRecycler::new(),
            t_hour: PrimitiveArrayRecycler::new(),
            t_minute: PrimitiveArrayRecycler::new(),
            t_second: PrimitiveArrayRecycler::new(),
            t_am_pm: StringViewArrayRecycler::new(),
            t_shift: StringViewArrayRecycler::new(),
            t_sub_shift: StringViewArrayRecycler::new(),
            t_meal_time: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for TimeDimArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for TimeDimArrow {
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
            t_time_sk,
            t_time_id,
            t_time,
            t_hour,
            t_minute,
            t_second,
            t_am_pm,
            t_shift,
            t_sub_shift,
            t_meal_time,
            ..
        } = self;
        let nbm = TimeDimRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&TimeDimRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&TimeDimRow) -> i32| {
            rec.int32(rows, pos, nbm, f)
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&TimeDimRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let t_time_sk = sk32(t_time_sk, 0, |r| r.t_time_sk);
        let t_time_id = str(t_time_id, 1, |r| r.t_time_id.as_str());
        let t_time = int(t_time, 2, |r| r.t_time);
        let t_hour = int(t_hour, 3, |r| r.t_hour);
        let t_minute = int(t_minute, 4, |r| r.t_minute);
        let t_second = int(t_second, 5, |r| r.t_second);
        let t_am_pm = str(t_am_pm, 6, |r| r.t_am_pm.as_str());
        let t_shift = str(t_shift, 7, |r| r.t_shift.as_str());
        let t_sub_shift = str(t_sub_shift, 8, |r| r.t_sub_shift.as_str());
        // t_meal_time is an empty string (not null) for hours with no meal,
        // but the pipe-delimited format can't distinguish empty from null,
        // so we map empty -> None to match the .dat file convention.
        let t_meal_time = t_meal_time.build(rows, |r| {
            (!r.t_meal_time.is_empty()).then_some(r.t_meal_time.as_str())
        });

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(t_time_sk),
                Arc::new(t_time_id),
                Arc::new(t_time),
                Arc::new(t_hour),
                Arc::new(t_minute),
                Arc::new(t_second),
                Arc::new(t_am_pm),
                Arc::new(t_shift),
                Arc::new(t_sub_shift),
                Arc::new(t_meal_time),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("t_time_sk", DataType::Int32, false),
        Field::new("t_time_id", DataType::Utf8View, false),
        Field::new("t_time", DataType::Int32, true),
        Field::new("t_hour", DataType::Int32, true),
        Field::new("t_minute", DataType::Int32, true),
        Field::new("t_second", DataType::Int32, true),
        Field::new("t_am_pm", DataType::Utf8View, true),
        Field::new("t_shift", DataType::Utf8View, true),
        Field::new("t_sub_shift", DataType::Utf8View, true),
        Field::new("t_meal_time", DataType::Utf8View, true),
    ]))
}
