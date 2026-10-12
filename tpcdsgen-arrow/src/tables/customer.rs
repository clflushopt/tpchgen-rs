use crate::conversions::{bool_to_yn, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerRow, CustomerRowGenerator};

/// Generate [`CustomerRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CustomerArrow {
    inner: CustomerRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CustomerRow>,
    c_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    c_customer_id: StringViewArrayRecycler,
    c_current_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    c_current_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    c_current_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    c_first_shipto_date_sk: PrimitiveArrayRecycler<Int32Type>,
    c_first_sales_date_sk: PrimitiveArrayRecycler<Int32Type>,
    c_salutation: StringViewArrayRecycler,
    c_first_name: StringViewArrayRecycler,
    c_last_name: StringViewArrayRecycler,
    c_preferred_cust_flag: StringViewArrayRecycler,
    c_birth_day: PrimitiveArrayRecycler<Int32Type>,
    c_birth_month: PrimitiveArrayRecycler<Int32Type>,
    c_birth_year: PrimitiveArrayRecycler<Int32Type>,
    c_birth_country: StringViewArrayRecycler,
    c_login: StringViewArrayRecycler,
    c_email_address: StringViewArrayRecycler,
    c_last_review_date_sk: PrimitiveArrayRecycler<Int32Type>,
}

impl CustomerArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Customer);
        Self {
            inner: CustomerRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            c_customer_sk: PrimitiveArrayRecycler::new(),
            c_customer_id: StringViewArrayRecycler::new(),
            c_current_cdemo_sk: PrimitiveArrayRecycler::new(),
            c_current_hdemo_sk: PrimitiveArrayRecycler::new(),
            c_current_addr_sk: PrimitiveArrayRecycler::new(),
            c_first_shipto_date_sk: PrimitiveArrayRecycler::new(),
            c_first_sales_date_sk: PrimitiveArrayRecycler::new(),
            c_salutation: StringViewArrayRecycler::new(),
            c_first_name: StringViewArrayRecycler::new(),
            c_last_name: StringViewArrayRecycler::new(),
            c_preferred_cust_flag: StringViewArrayRecycler::new(),
            c_birth_day: PrimitiveArrayRecycler::new(),
            c_birth_month: PrimitiveArrayRecycler::new(),
            c_birth_year: PrimitiveArrayRecycler::new(),
            c_birth_country: StringViewArrayRecycler::new(),
            c_login: StringViewArrayRecycler::new(),
            c_email_address: StringViewArrayRecycler::new(),
            c_last_review_date_sk: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for CustomerArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CustomerArrow {
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
            c_customer_sk,
            c_customer_id,
            c_current_cdemo_sk,
            c_current_hdemo_sk,
            c_current_addr_sk,
            c_first_shipto_date_sk,
            c_first_sales_date_sk,
            c_salutation,
            c_first_name,
            c_last_name,
            c_preferred_cust_flag,
            c_birth_day,
            c_birth_month,
            c_birth_year,
            c_birth_country,
            c_login,
            c_email_address,
            c_last_review_date_sk,
            ..
        } = self;
        let nbm = CustomerRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CustomerRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CustomerRow) -> i32| {
            rec.int32(rows, pos, nbm, f)
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&CustomerRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let c_customer_sk = sk32(c_customer_sk, 0, CustomerRow::get_c_customer_sk);
        let c_customer_id = str(c_customer_id, 1, CustomerRow::get_c_customer_id);
        let c_current_cdemo_sk = sk32(c_current_cdemo_sk, 2, CustomerRow::get_c_current_cdemo_sk);
        let c_current_hdemo_sk = sk32(c_current_hdemo_sk, 3, CustomerRow::get_c_current_hdemo_sk);
        let c_current_addr_sk = sk32(c_current_addr_sk, 4, CustomerRow::get_c_current_addr_sk);
        let c_first_shipto_date_sk = int(
            c_first_shipto_date_sk,
            5,
            CustomerRow::get_c_first_shipto_date_id,
        );
        let c_first_sales_date_sk = int(
            c_first_sales_date_sk,
            6,
            CustomerRow::get_c_first_sales_date_id,
        );
        let c_salutation = str(c_salutation, 7, CustomerRow::get_c_salutation);
        let c_first_name = str(c_first_name, 8, CustomerRow::get_c_first_name);
        let c_last_name = str(c_last_name, 9, CustomerRow::get_c_last_name);
        let c_preferred_cust_flag = str(c_preferred_cust_flag, 10, |r| {
            bool_to_yn(r.get_c_preferred_cust_flag())
        });
        let c_birth_day = int(c_birth_day, 11, CustomerRow::get_c_birth_day);
        let c_birth_month = int(c_birth_month, 12, CustomerRow::get_c_birth_month);
        let c_birth_year = int(c_birth_year, 13, CustomerRow::get_c_birth_year);
        let c_birth_country = str(c_birth_country, 14, CustomerRow::get_c_birth_country);
        // always null per TPC-DS spec
        let c_login = c_login.build(rows, |_| None);
        let c_email_address = str(c_email_address, 16, CustomerRow::get_c_email_address);
        let c_last_review_date_sk = int(
            c_last_review_date_sk,
            17,
            CustomerRow::get_c_last_review_date,
        );

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(c_customer_sk),
                Arc::new(c_customer_id),
                Arc::new(c_current_cdemo_sk),
                Arc::new(c_current_hdemo_sk),
                Arc::new(c_current_addr_sk),
                Arc::new(c_first_shipto_date_sk),
                Arc::new(c_first_sales_date_sk),
                Arc::new(c_salutation),
                Arc::new(c_first_name),
                Arc::new(c_last_name),
                Arc::new(c_preferred_cust_flag),
                Arc::new(c_birth_day),
                Arc::new(c_birth_month),
                Arc::new(c_birth_year),
                Arc::new(c_birth_country),
                Arc::new(c_login),
                Arc::new(c_email_address),
                Arc::new(c_last_review_date_sk),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("c_customer_sk", DataType::Int32, false),
        Field::new("c_customer_id", DataType::Utf8View, false),
        Field::new("c_current_cdemo_sk", DataType::Int32, true),
        Field::new("c_current_hdemo_sk", DataType::Int32, true),
        Field::new("c_current_addr_sk", DataType::Int32, true),
        Field::new("c_first_shipto_date_sk", DataType::Int32, true),
        Field::new("c_first_sales_date_sk", DataType::Int32, true),
        Field::new("c_salutation", DataType::Utf8View, true),
        Field::new("c_first_name", DataType::Utf8View, true),
        Field::new("c_last_name", DataType::Utf8View, true),
        Field::new("c_preferred_cust_flag", DataType::Utf8View, true),
        Field::new("c_birth_day", DataType::Int32, true),
        Field::new("c_birth_month", DataType::Int32, true),
        Field::new("c_birth_year", DataType::Int32, true),
        Field::new("c_birth_country", DataType::Utf8View, true),
        Field::new("c_login", DataType::Utf8View, true),
        Field::new("c_email_address", DataType::Utf8View, true),
        Field::new("c_last_review_date_sk", DataType::Int32, true),
    ]))
}
