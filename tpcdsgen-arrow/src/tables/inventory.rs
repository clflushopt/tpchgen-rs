use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{InventoryRow, InventoryRowGenerator};

/// Generate [`InventoryRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct InventoryArrow {
    inner: InventoryRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<InventoryRow>,
    inv_date_sk: PrimitiveArrayRecycler<Int32Type>,
    inv_item_sk: PrimitiveArrayRecycler<Int32Type>,
    inv_warehouse_sk: PrimitiveArrayRecycler<Int32Type>,
    inv_quantity_on_hand: PrimitiveArrayRecycler<Int32Type>,
}

impl InventoryArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        Self {
            inner: InventoryRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            inv_date_sk: PrimitiveArrayRecycler::new(),
            inv_item_sk: PrimitiveArrayRecycler::new(),
            inv_warehouse_sk: PrimitiveArrayRecycler::new(),
            inv_quantity_on_hand: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for InventoryArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for InventoryArrow {
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
            inv_date_sk,
            inv_item_sk,
            inv_warehouse_sk,
            inv_quantity_on_hand,
            ..
        } = self;
        let nbm = InventoryRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&InventoryRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };

        let inv_date_sk = sk32(inv_date_sk, 0, InventoryRow::get_inv_date_sk);
        let inv_item_sk = sk32(inv_item_sk, 1, InventoryRow::get_inv_item_sk);
        let inv_warehouse_sk = sk32(inv_warehouse_sk, 2, InventoryRow::get_inv_warehouse_sk);
        let inv_quantity_on_hand =
            inv_quantity_on_hand.int32(rows, 3, nbm, InventoryRow::get_inv_quantity_on_hand);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(inv_date_sk),
                Arc::new(inv_item_sk),
                Arc::new(inv_warehouse_sk),
                Arc::new(inv_quantity_on_hand),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("inv_date_sk", DataType::Int32, false),
        Field::new("inv_item_sk", DataType::Int32, false),
        Field::new("inv_warehouse_sk", DataType::Int32, false),
        Field::new("inv_quantity_on_hand", DataType::Int32, true),
    ]))
}
