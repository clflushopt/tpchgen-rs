use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{ReasonRow, ReasonRowGenerator};

/// Generate [`ReasonRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct ReasonArrow {
    inner: ReasonRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<ReasonRow>,
    r_reason_sk: PrimitiveArrayRecycler<Int32Type>,
    r_reason_id: StringViewArrayRecycler,
    r_reason_desc: StringViewArrayRecycler,
}

impl ReasonArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Reason);
        Self {
            inner: ReasonRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            r_reason_sk: PrimitiveArrayRecycler::new(),
            r_reason_id: StringViewArrayRecycler::new(),
            r_reason_desc: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for ReasonArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for ReasonArrow {
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
            r_reason_sk,
            r_reason_id,
            r_reason_desc,
            ..
        } = self;
        let nbm = ReasonRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&ReasonRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&ReasonRow) -> &str| {
            rec.string(rows, pos, nbm, f)
        };

        let r_reason_sk = sk32(r_reason_sk, 0, ReasonRow::get_r_reason_sk);
        let r_reason_id = str(r_reason_id, 1, ReasonRow::get_r_reason_id);
        let r_reason_desc = str(r_reason_desc, 2, ReasonRow::get_r_reason_desc);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(r_reason_sk),
                Arc::new(r_reason_id),
                Arc::new(r_reason_desc),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("r_reason_sk", DataType::Int32, false),
        Field::new("r_reason_id", DataType::Utf8View, false),
        Field::new("r_reason_desc", DataType::Utf8View, true),
    ]))
}
