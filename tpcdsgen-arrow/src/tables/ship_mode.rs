use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{ShipModeRow, ShipModeRowGenerator};

/// Generate [`ShipModeRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct ShipModeArrow {
    inner: ShipModeRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<ShipModeRow>,
    sm_ship_mode_sk: PrimitiveArrayRecycler<Int32Type>,
    sm_ship_mode_id: StringViewArrayRecycler,
    sm_type: StringViewArrayRecycler,
    sm_code: StringViewArrayRecycler,
    sm_carrier: StringViewArrayRecycler,
    sm_contract: StringViewArrayRecycler,
}

impl ShipModeArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::ShipMode);
        Self {
            inner: ShipModeRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            sm_ship_mode_sk: PrimitiveArrayRecycler::new(),
            sm_ship_mode_id: StringViewArrayRecycler::new(),
            sm_type: StringViewArrayRecycler::new(),
            sm_code: StringViewArrayRecycler::new(),
            sm_carrier: StringViewArrayRecycler::new(),
            sm_contract: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for ShipModeArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for ShipModeArrow {
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
            sm_ship_mode_sk,
            sm_ship_mode_id,
            sm_type,
            sm_code,
            sm_carrier,
            sm_contract,
            ..
        } = self;
        let nbm = ShipModeRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&ShipModeRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&ShipModeRow) -> &str| {
            rec.string(rows, pos, nbm, f)
        };

        let sm_ship_mode_sk = sk32(sm_ship_mode_sk, 0, ShipModeRow::get_sm_ship_mode_sk);
        let sm_ship_mode_id = str(sm_ship_mode_id, 1, ShipModeRow::get_sm_ship_mode_id);
        let sm_type = str(sm_type, 2, ShipModeRow::get_sm_type);
        let sm_code = str(sm_code, 3, ShipModeRow::get_sm_code);
        let sm_carrier = str(sm_carrier, 4, ShipModeRow::get_sm_carrier);
        let sm_contract = str(sm_contract, 5, ShipModeRow::get_sm_contract);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(sm_ship_mode_sk),
                Arc::new(sm_ship_mode_id),
                Arc::new(sm_type),
                Arc::new(sm_code),
                Arc::new(sm_carrier),
                Arc::new(sm_contract),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sm_ship_mode_sk", DataType::Int32, false),
        Field::new("sm_ship_mode_id", DataType::Utf8View, false),
        Field::new("sm_type", DataType::Utf8View, true),
        Field::new("sm_code", DataType::Utf8View, true),
        Field::new("sm_carrier", DataType::Utf8View, true),
        Field::new("sm_contract", DataType::Utf8View, true),
    ]))
}
