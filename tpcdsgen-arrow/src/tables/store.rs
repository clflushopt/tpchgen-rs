use crate::conversions::{AddressArrayRecycler, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Decimal128Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{StoreRow, StoreRowGenerator};

/// Generate [`StoreRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct StoreArrow {
    inner: StoreRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<StoreRow>,
    s_store_sk: PrimitiveArrayRecycler<Int32Type>,
    s_store_id: StringViewArrayRecycler,
    s_rec_start_date: PrimitiveArrayRecycler<Date32Type>,
    s_rec_end_date: PrimitiveArrayRecycler<Date32Type>,
    s_closed_date_sk: PrimitiveArrayRecycler<Int32Type>,
    s_store_name: StringViewArrayRecycler,
    s_number_employees: PrimitiveArrayRecycler<Int32Type>,
    s_floor_space: PrimitiveArrayRecycler<Int32Type>,
    s_hours: StringViewArrayRecycler,
    s_manager: StringViewArrayRecycler,
    s_market_id: PrimitiveArrayRecycler<Int32Type>,
    s_geography_class: StringViewArrayRecycler,
    s_market_desc: StringViewArrayRecycler,
    s_market_manager: StringViewArrayRecycler,
    s_division_id: PrimitiveArrayRecycler<Int32Type>,
    s_division_name: StringViewArrayRecycler,
    s_company_id: PrimitiveArrayRecycler<Int32Type>,
    s_company_name: StringViewArrayRecycler,
    address: AddressArrayRecycler,
    s_tax_precentage: PrimitiveArrayRecycler<Decimal128Type>,
}

impl StoreArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Store);
        Self {
            inner: StoreRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            s_store_sk: PrimitiveArrayRecycler::new(),
            s_store_id: StringViewArrayRecycler::new(),
            s_rec_start_date: PrimitiveArrayRecycler::new(),
            s_rec_end_date: PrimitiveArrayRecycler::new(),
            s_closed_date_sk: PrimitiveArrayRecycler::new(),
            s_store_name: StringViewArrayRecycler::new(),
            s_number_employees: PrimitiveArrayRecycler::new(),
            s_floor_space: PrimitiveArrayRecycler::new(),
            s_hours: StringViewArrayRecycler::new(),
            s_manager: StringViewArrayRecycler::new(),
            s_market_id: PrimitiveArrayRecycler::new(),
            s_geography_class: StringViewArrayRecycler::new(),
            s_market_desc: StringViewArrayRecycler::new(),
            s_market_manager: StringViewArrayRecycler::new(),
            s_division_id: PrimitiveArrayRecycler::new(),
            s_division_name: StringViewArrayRecycler::new(),
            s_company_id: PrimitiveArrayRecycler::new(),
            s_company_name: StringViewArrayRecycler::new(),
            address: AddressArrayRecycler::new(),
            s_tax_precentage: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for StoreArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for StoreArrow {
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
            s_store_sk,
            s_store_id,
            s_rec_start_date,
            s_rec_end_date,
            s_closed_date_sk,
            s_store_name,
            s_number_employees,
            s_floor_space,
            s_hours,
            s_manager,
            s_market_id,
            s_geography_class,
            s_market_desc,
            s_market_manager,
            s_division_id,
            s_division_name,
            s_company_id,
            s_company_name,
            address,
            s_tax_precentage,
            ..
        } = self;
        let nbm = StoreRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&StoreRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&StoreRow) -> i32| {
            rec.int32(rows, pos, nbm, f)
        };
        let int64 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&StoreRow) -> i64| {
            rec.int32(rows, pos, nbm, |r| {
                i32::try_from(f(r)).expect("TPC-DS INTEGER value exceeds i32 range")
            })
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&StoreRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };
        let date = |rec: &mut PrimitiveArrayRecycler<Date32Type>, pos, f: fn(&StoreRow) -> i64| {
            rec.date32_julian(rows, pos, nbm, f)
        };

        let s_store_sk = sk32(s_store_sk, 0, StoreRow::get_store_sk);
        let s_store_id = str(s_store_id, 1, StoreRow::get_store_id);
        let s_rec_start_date = date(s_rec_start_date, 2, StoreRow::get_rec_start_date_id);
        // only the negative-Julian sentinel marks null here, not the bitmap
        let s_rec_end_date =
            s_rec_end_date.date32_julian(rows, 3, |_| 0, StoreRow::get_rec_end_date_id);
        let s_closed_date_sk = sk32(s_closed_date_sk, 4, StoreRow::get_closed_date_id);
        let s_store_name = str(s_store_name, 5, StoreRow::get_store_name);
        let s_number_employees = int(s_number_employees, 6, StoreRow::get_employees);
        let s_floor_space = int(s_floor_space, 7, StoreRow::get_floor_space);
        let s_hours = str(s_hours, 8, StoreRow::get_hours);
        let s_manager = str(s_manager, 9, StoreRow::get_store_manager);
        let s_market_id = int(s_market_id, 10, StoreRow::get_market_id);
        // Null bits follow the generator column numbering, where the tax
        // percentage (bit 11) comes before geography class, even though it
        // is the last output column.
        let s_geography_class = str(s_geography_class, 12, StoreRow::get_geography_class);
        let s_market_desc = str(s_market_desc, 13, StoreRow::get_market_desc);
        let s_market_manager = str(s_market_manager, 14, StoreRow::get_market_manager);
        let s_division_id = int64(s_division_id, 15, StoreRow::get_division_id);
        let s_division_name = str(s_division_name, 16, StoreRow::get_division_name);
        let s_company_id = int64(s_company_id, 17, StoreRow::get_company_id);
        let s_company_name = str(s_company_name, 18, StoreRow::get_company_name);
        let address = address.build(rows, 19, nbm, StoreRow::get_address);
        let s_tax_precentage =
            s_tax_precentage.decimal_5_2(rows, 11, nbm, StoreRow::get_d_tax_percentage);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(s_store_sk),
                Arc::new(s_store_id),
                Arc::new(s_rec_start_date),
                Arc::new(s_rec_end_date),
                Arc::new(s_closed_date_sk),
                Arc::new(s_store_name),
                Arc::new(s_number_employees),
                Arc::new(s_floor_space),
                Arc::new(s_hours),
                Arc::new(s_manager),
                Arc::new(s_market_id),
                Arc::new(s_geography_class),
                Arc::new(s_market_desc),
                Arc::new(s_market_manager),
                Arc::new(s_division_id),
                Arc::new(s_division_name),
                Arc::new(s_company_id),
                Arc::new(s_company_name),
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
                Arc::new(s_tax_precentage),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("s_store_sk", DataType::Int32, false),
        Field::new("s_store_id", DataType::Utf8View, false),
        Field::new("s_rec_start_date", DataType::Date32, true),
        Field::new("s_rec_end_date", DataType::Date32, true),
        Field::new("s_closed_date_sk", DataType::Int32, true),
        Field::new("s_store_name", DataType::Utf8View, true),
        Field::new("s_number_employees", DataType::Int32, true),
        Field::new("s_floor_space", DataType::Int32, true),
        Field::new("s_hours", DataType::Utf8View, true),
        Field::new("s_manager", DataType::Utf8View, true),
        Field::new("s_market_id", DataType::Int32, true),
        Field::new("s_geography_class", DataType::Utf8View, true),
        Field::new("s_market_desc", DataType::Utf8View, true),
        Field::new("s_market_manager", DataType::Utf8View, true),
        Field::new("s_division_id", DataType::Int32, true),
        Field::new("s_division_name", DataType::Utf8View, true),
        Field::new("s_company_id", DataType::Int32, true),
        Field::new("s_company_name", DataType::Utf8View, true),
        Field::new("s_street_number", DataType::Utf8View, true),
        Field::new("s_street_name", DataType::Utf8View, true),
        Field::new("s_street_type", DataType::Utf8View, true),
        Field::new("s_suite_number", DataType::Utf8View, true),
        Field::new("s_city", DataType::Utf8View, true),
        Field::new("s_county", DataType::Utf8View, true),
        Field::new("s_state", DataType::Utf8View, true),
        Field::new("s_zip", DataType::Utf8View, true),
        Field::new("s_country", DataType::Utf8View, true),
        Field::new("s_gmt_offset", DataType::Decimal128(5, 2), true),
        Field::new("s_tax_precentage", DataType::Decimal128(5, 2), true),
    ]))
}
