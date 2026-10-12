use crate::conversions::{AddressArrayRecycler, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Decimal128Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CallCenterRow, CallCenterRowGenerator};

/// Generate [`CallCenterRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CallCenterArrow {
    inner: CallCenterRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CallCenterRow>,
    cc_call_center_sk: PrimitiveArrayRecycler<Int32Type>,
    cc_call_center_id: StringViewArrayRecycler,
    cc_rec_start_date: PrimitiveArrayRecycler<Date32Type>,
    cc_rec_end_date: PrimitiveArrayRecycler<Date32Type>,
    cc_closed_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cc_open_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cc_name: StringViewArrayRecycler,
    cc_class: StringViewArrayRecycler,
    cc_employees: PrimitiveArrayRecycler<Int32Type>,
    cc_sq_ft: PrimitiveArrayRecycler<Int32Type>,
    cc_hours: StringViewArrayRecycler,
    cc_manager: StringViewArrayRecycler,
    cc_mkt_id: PrimitiveArrayRecycler<Int32Type>,
    cc_mkt_class: StringViewArrayRecycler,
    cc_mkt_desc: StringViewArrayRecycler,
    cc_market_manager: StringViewArrayRecycler,
    cc_division: PrimitiveArrayRecycler<Int32Type>,
    cc_division_name: StringViewArrayRecycler,
    cc_company: PrimitiveArrayRecycler<Int32Type>,
    cc_company_name: StringViewArrayRecycler,
    address: AddressArrayRecycler,
    cc_tax_percentage: PrimitiveArrayRecycler<Decimal128Type>,
}

impl CallCenterArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CallCenter);
        Self {
            inner: CallCenterRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            cc_call_center_sk: PrimitiveArrayRecycler::new(),
            cc_call_center_id: StringViewArrayRecycler::new(),
            cc_rec_start_date: PrimitiveArrayRecycler::new(),
            cc_rec_end_date: PrimitiveArrayRecycler::new(),
            cc_closed_date_sk: PrimitiveArrayRecycler::new(),
            cc_open_date_sk: PrimitiveArrayRecycler::new(),
            cc_name: StringViewArrayRecycler::new(),
            cc_class: StringViewArrayRecycler::new(),
            cc_employees: PrimitiveArrayRecycler::new(),
            cc_sq_ft: PrimitiveArrayRecycler::new(),
            cc_hours: StringViewArrayRecycler::new(),
            cc_manager: StringViewArrayRecycler::new(),
            cc_mkt_id: PrimitiveArrayRecycler::new(),
            cc_mkt_class: StringViewArrayRecycler::new(),
            cc_mkt_desc: StringViewArrayRecycler::new(),
            cc_market_manager: StringViewArrayRecycler::new(),
            cc_division: PrimitiveArrayRecycler::new(),
            cc_division_name: StringViewArrayRecycler::new(),
            cc_company: PrimitiveArrayRecycler::new(),
            cc_company_name: StringViewArrayRecycler::new(),
            address: AddressArrayRecycler::new(),
            cc_tax_percentage: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for CallCenterArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CallCenterArrow {
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
            cc_call_center_sk,
            cc_call_center_id,
            cc_rec_start_date,
            cc_rec_end_date,
            cc_closed_date_sk,
            cc_open_date_sk,
            cc_name,
            cc_class,
            cc_employees,
            cc_sq_ft,
            cc_hours,
            cc_manager,
            cc_mkt_id,
            cc_mkt_class,
            cc_mkt_desc,
            cc_market_manager,
            cc_division,
            cc_division_name,
            cc_company,
            cc_company_name,
            address,
            cc_tax_percentage,
            ..
        } = self;
        let nbm = CallCenterRow::get_null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CallCenterRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>,
                   pos,
                   f: fn(&CallCenterRow) -> i32| { rec.int32(rows, pos, nbm, f) };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&CallCenterRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };
        let date =
            |rec: &mut PrimitiveArrayRecycler<Date32Type>, pos, f: fn(&CallCenterRow) -> i64| {
                rec.date32_julian(rows, pos, nbm, f)
            };

        let cc_call_center_sk = sk32(cc_call_center_sk, 0, CallCenterRow::get_cc_call_center_sk);
        let cc_call_center_id = str(cc_call_center_id, 1, CallCenterRow::get_cc_call_center_id);
        let cc_rec_start_date = date(
            cc_rec_start_date,
            2,
            CallCenterRow::get_cc_rec_start_date_id,
        );
        let cc_rec_end_date = date(cc_rec_end_date, 3, CallCenterRow::get_cc_rec_end_date_id);
        let cc_closed_date_sk = sk32(cc_closed_date_sk, 4, CallCenterRow::get_cc_closed_date_id);
        let cc_open_date_sk = sk32(cc_open_date_sk, 5, CallCenterRow::get_cc_open_date_id);
        let cc_name = str(cc_name, 6, CallCenterRow::get_cc_name);
        let cc_class = str(cc_class, 7, CallCenterRow::get_cc_class);
        let cc_employees = int(cc_employees, 8, CallCenterRow::get_cc_employees);
        let cc_sq_ft = int(cc_sq_ft, 9, CallCenterRow::get_cc_sq_ft);
        let cc_hours = str(cc_hours, 10, CallCenterRow::get_cc_hours);
        let cc_manager = str(cc_manager, 11, CallCenterRow::get_cc_manager);
        let cc_mkt_id = int(cc_mkt_id, 12, CallCenterRow::get_cc_market_id);
        let cc_mkt_class = str(cc_mkt_class, 13, CallCenterRow::get_cc_market_class);
        let cc_mkt_desc = str(cc_mkt_desc, 14, CallCenterRow::get_cc_market_desc);
        let cc_market_manager = str(cc_market_manager, 15, CallCenterRow::get_cc_market_manager);
        let cc_division = int(cc_division, 16, CallCenterRow::get_cc_division_id);
        let cc_division_name = str(cc_division_name, 17, CallCenterRow::get_cc_division_name);
        let cc_company = int(cc_company, 18, CallCenterRow::get_cc_company);
        let cc_company_name = str(cc_company_name, 19, CallCenterRow::get_cc_company_name);
        let address = address.build(rows, 20, nbm, CallCenterRow::get_cc_address);
        let cc_tax_percentage =
            cc_tax_percentage.decimal_5_2(rows, 30, nbm, |r| *r.get_cc_tax_percentage());

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(cc_call_center_sk),
                Arc::new(cc_call_center_id),
                Arc::new(cc_rec_start_date),
                Arc::new(cc_rec_end_date),
                Arc::new(cc_closed_date_sk),
                Arc::new(cc_open_date_sk),
                Arc::new(cc_name),
                Arc::new(cc_class),
                Arc::new(cc_employees),
                Arc::new(cc_sq_ft),
                Arc::new(cc_hours),
                Arc::new(cc_manager),
                Arc::new(cc_mkt_id),
                Arc::new(cc_mkt_class),
                Arc::new(cc_mkt_desc),
                Arc::new(cc_market_manager),
                Arc::new(cc_division),
                Arc::new(cc_division_name),
                Arc::new(cc_company),
                Arc::new(cc_company_name),
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
                Arc::new(cc_tax_percentage),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cc_call_center_sk", DataType::Int32, false),
        Field::new("cc_call_center_id", DataType::Utf8View, false),
        Field::new("cc_rec_start_date", DataType::Date32, true),
        Field::new("cc_rec_end_date", DataType::Date32, true),
        Field::new("cc_closed_date_sk", DataType::Int32, true),
        Field::new("cc_open_date_sk", DataType::Int32, true),
        Field::new("cc_name", DataType::Utf8View, true),
        Field::new("cc_class", DataType::Utf8View, true),
        Field::new("cc_employees", DataType::Int32, true),
        Field::new("cc_sq_ft", DataType::Int32, true),
        Field::new("cc_hours", DataType::Utf8View, true),
        Field::new("cc_manager", DataType::Utf8View, true),
        Field::new("cc_mkt_id", DataType::Int32, true),
        Field::new("cc_mkt_class", DataType::Utf8View, true),
        Field::new("cc_mkt_desc", DataType::Utf8View, true),
        Field::new("cc_market_manager", DataType::Utf8View, true),
        Field::new("cc_division", DataType::Int32, true),
        Field::new("cc_division_name", DataType::Utf8View, true),
        Field::new("cc_company", DataType::Int32, true),
        Field::new("cc_company_name", DataType::Utf8View, true),
        Field::new("cc_street_number", DataType::Utf8View, true),
        Field::new("cc_street_name", DataType::Utf8View, true),
        Field::new("cc_street_type", DataType::Utf8View, true),
        Field::new("cc_suite_number", DataType::Utf8View, true),
        Field::new("cc_city", DataType::Utf8View, true),
        Field::new("cc_county", DataType::Utf8View, true),
        Field::new("cc_state", DataType::Utf8View, true),
        Field::new("cc_zip", DataType::Utf8View, true),
        Field::new("cc_country", DataType::Utf8View, true),
        Field::new("cc_gmt_offset", DataType::Decimal128(5, 2), true),
        Field::new("cc_tax_percentage", DataType::Decimal128(5, 2), true),
    ]))
}
