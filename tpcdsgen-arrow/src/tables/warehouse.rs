use crate::conversions::{AddressArrayRecycler, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{WarehouseRow, WarehouseRowGenerator};

/// Generate [`WarehouseRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct WarehouseArrow {
    inner: WarehouseRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<WarehouseRow>,
    w_warehouse_sk: PrimitiveArrayRecycler<Int32Type>,
    w_warehouse_id: StringViewArrayRecycler,
    w_warehouse_name: StringViewArrayRecycler,
    w_warehouse_sq_ft: PrimitiveArrayRecycler<Int32Type>,
    address: AddressArrayRecycler,
}

impl WarehouseArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Warehouse);
        Self {
            inner: WarehouseRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            w_warehouse_sk: PrimitiveArrayRecycler::new(),
            w_warehouse_id: StringViewArrayRecycler::new(),
            w_warehouse_name: StringViewArrayRecycler::new(),
            w_warehouse_sq_ft: PrimitiveArrayRecycler::new(),
            address: AddressArrayRecycler::new(),
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

impl RecordBatchReader for WarehouseArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WarehouseArrow {
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
            w_warehouse_sk,
            w_warehouse_id,
            w_warehouse_name,
            w_warehouse_sq_ft,
            address,
            ..
        } = self;
        let nbm = WarehouseRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WarehouseRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>,
                   pos,
                   f: fn(&WarehouseRow) -> i32| { rec.int32(rows, pos, nbm, f) };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&WarehouseRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let w_warehouse_sk = sk32(w_warehouse_sk, 0, WarehouseRow::get_w_warehouse_sk);
        let w_warehouse_id = str(w_warehouse_id, 1, WarehouseRow::get_w_warehouse_id);
        let w_warehouse_name = str(w_warehouse_name, 2, WarehouseRow::get_w_warehouse_name);
        let w_warehouse_sq_ft = int(w_warehouse_sq_ft, 3, WarehouseRow::get_w_warehouse_sq_ft);
        let address = address.build(rows, 4, nbm, WarehouseRow::get_w_address);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(w_warehouse_sk),
                Arc::new(w_warehouse_id),
                Arc::new(w_warehouse_name),
                Arc::new(w_warehouse_sq_ft),
                Arc::new(address.street_number),
                Arc::new(address.street_name),
                Arc::new(address.street_type),
                Arc::new(address.suite_number),
                Arc::new(address.city),
                Arc::new(address.county),
                Arc::new(address.state),
                Arc::new(address.zip),
                Arc::new(address.country),
                Arc::new(address.gmt_offset),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("w_warehouse_sk", DataType::Int32, false),
        Field::new("w_warehouse_id", DataType::Utf8View, false),
        Field::new("w_warehouse_name", DataType::Utf8View, true),
        Field::new("w_warehouse_sq_ft", DataType::Int32, true),
        Field::new("w_street_number", DataType::Utf8View, true),
        Field::new("w_street_name", DataType::Utf8View, true),
        Field::new("w_street_type", DataType::Utf8View, true),
        Field::new("w_suite_number", DataType::Utf8View, true),
        Field::new("w_city", DataType::Utf8View, true),
        Field::new("w_county", DataType::Utf8View, true),
        Field::new("w_state", DataType::Utf8View, true),
        Field::new("w_zip", DataType::Utf8View, true),
        Field::new("w_country", DataType::Utf8View, true),
        Field::new("w_gmt_offset", DataType::Decimal128(5, 2), true),
    ]))
}
