use crate::conversions::{AddressArrayRecycler, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerAddressRow, CustomerAddressRowGenerator};

/// Generate [`CustomerAddressRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CustomerAddressArrow {
    inner: CustomerAddressRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CustomerAddressRow>,
    ca_address_sk: PrimitiveArrayRecycler<Int32Type>,
    ca_address_id: StringViewArrayRecycler,
    ca_address: AddressArrayRecycler,
    ca_location_type: StringViewArrayRecycler,
}

impl CustomerAddressArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CustomerAddress);
        Self {
            inner: CustomerAddressRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            ca_address_sk: PrimitiveArrayRecycler::new(),
            ca_address_id: StringViewArrayRecycler::new(),
            ca_address: AddressArrayRecycler::new(),
            ca_location_type: StringViewArrayRecycler::new(),
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

impl RecordBatchReader for CustomerAddressArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CustomerAddressArrow {
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
            ca_address_sk,
            ca_address_id,
            ca_address,
            ca_location_type,
            ..
        } = self;
        let nbm = CustomerAddressRow::null_bit_map;

        let ca_address_sk =
            ca_address_sk.int32_sk(rows, 0, nbm, CustomerAddressRow::get_ca_addr_sk);
        let ca_address_id = ca_address_id.str(rows, 1, nbm, CustomerAddressRow::get_ca_addr_id);
        let address = ca_address.build(rows, 2, nbm, CustomerAddressRow::get_ca_address);
        let ca_location_type =
            ca_location_type.str(rows, 12, nbm, CustomerAddressRow::get_ca_location_type);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ca_address_sk),
                Arc::new(ca_address_id),
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
                Arc::new(ca_location_type),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ca_address_sk", DataType::Int32, false),
        Field::new("ca_address_id", DataType::Utf8View, false),
        Field::new("ca_street_number", DataType::Utf8View, true),
        Field::new("ca_street_name", DataType::Utf8View, true),
        Field::new("ca_street_type", DataType::Utf8View, true),
        Field::new("ca_suite_number", DataType::Utf8View, true),
        Field::new("ca_city", DataType::Utf8View, true),
        Field::new("ca_county", DataType::Utf8View, true),
        Field::new("ca_state", DataType::Utf8View, true),
        Field::new("ca_zip", DataType::Utf8View, true),
        Field::new("ca_country", DataType::Utf8View, true),
        Field::new("ca_gmt_offset", DataType::Decimal128(5, 2), true),
        Field::new("ca_location_type", DataType::Utf8View, true),
    ]))
}
