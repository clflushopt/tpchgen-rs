use crate::conversions::{is_null, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{
    DataType, Date32Type, Field, Schema, SchemaRef, Time32SecondType, TimeUnit,
};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DbgenVersionRow, DbgenVersionRowGenerator};

/// Generate [`DbgenVersionRow`]s in [`RecordBatch`] format
pub struct DbgenVersionArrow {
    inner: DbgenVersionRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<DbgenVersionRow>,
    dv_version: StringViewArrayRecycler,
    dv_create_date: PrimitiveArrayRecycler<Date32Type>,
    dv_create_time: PrimitiveArrayRecycler<Time32SecondType>,
    dv_cmdline_args: StringViewArrayRecycler,
}

impl DbgenVersionArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DbgenVersion);
        Self {
            inner: DbgenVersionRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            dv_version: StringViewArrayRecycler::new(),
            dv_create_date: PrimitiveArrayRecycler::new(),
            dv_create_time: PrimitiveArrayRecycler::new(),
            dv_cmdline_args: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for DbgenVersionArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for DbgenVersionArrow {
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
            dv_version,
            dv_create_date,
            dv_create_time,
            dv_cmdline_args,
            ..
        } = self;
        let nbm = DbgenVersionRow::null_bit_map;

        let dv_version = dv_version.str(rows, 0, nbm, DbgenVersionRow::get_dv_version);
        let dv_create_date = dv_create_date.date32(rows, 1, nbm, |r| *r.get_dv_create_date());
        let dv_create_time = dv_create_time.build(rows, DbgenVersionRow::get_dv_create_time, |r| {
            !is_null(nbm(r), 2)
        });
        let dv_cmdline_args =
            dv_cmdline_args.str(rows, 3, nbm, DbgenVersionRow::get_dv_cmdline_args);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(dv_version),
                Arc::new(dv_create_date),
                Arc::new(dv_create_time),
                Arc::new(dv_cmdline_args),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("dv_version", DataType::Utf8View, true),
        Field::new("dv_create_date", DataType::Date32, true),
        Field::new("dv_create_time", DataType::Time32(TimeUnit::Second), true),
        Field::new("dv_cmdline_args", DataType::Utf8View, true),
    ]))
}
