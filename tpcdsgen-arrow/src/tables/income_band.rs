use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{IncomeBandRow, IncomeBandRowGenerator};

/// Generate [`IncomeBandRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct IncomeBandArrow {
    inner: IncomeBandRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<IncomeBandRow>,
    ib_income_band_sk: PrimitiveArrayRecycler<Int32Type>,
    ib_lower_bound: PrimitiveArrayRecycler<Int32Type>,
    ib_upper_bound: PrimitiveArrayRecycler<Int32Type>,
}

impl IncomeBandArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::IncomeBand);
        Self {
            inner: IncomeBandRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            ib_income_band_sk: PrimitiveArrayRecycler::new(),
            ib_lower_bound: PrimitiveArrayRecycler::new(),
            ib_upper_bound: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for IncomeBandArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for IncomeBandArrow {
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
            ib_income_band_sk,
            ib_lower_bound,
            ib_upper_bound,
            ..
        } = self;
        let nbm = IncomeBandRow::null_bit_map;

        let ib_income_band_sk =
            ib_income_band_sk.int32(rows, 0, nbm, IncomeBandRow::get_ib_income_band_sk);
        let ib_lower_bound = ib_lower_bound.int32(rows, 1, nbm, IncomeBandRow::get_ib_lower_bound);
        let ib_upper_bound = ib_upper_bound.int32(rows, 2, nbm, IncomeBandRow::get_ib_upper_bound);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ib_income_band_sk),
                Arc::new(ib_lower_bound),
                Arc::new(ib_upper_bound),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ib_income_band_sk", DataType::Int32, false),
        Field::new("ib_lower_bound", DataType::Int32, true),
        Field::new("ib_upper_bound", DataType::Int32, true),
    ]))
}
